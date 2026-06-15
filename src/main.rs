use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    process::Command,
};

use anyhow::{Context, Result, anyhow, bail};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

const CLIENT_NAME: &str = "embycli";
const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)]
#[command(
    name = "embycli",
    version,
    about = "Search and play media from an Emby server"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Log in to an Emby server and save the access token locally.
    Login {
        /// Emby server base URL, for example http://127.0.0.1:8096.
        server: String,

        /// Emby username.
        #[arg(short, long)]
        username: String,

        /// Password. Omit this option to enter it interactively.
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Show the saved server and user.
    Whoami,

    /// Search movies, series, episodes, music videos, and videos.
    Search {
        /// Search text.
        query: String,

        /// Maximum number of results.
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
    },

    /// Play an item by id, or search text and play the selected result.
    Play {
        /// Item id or search text.
        target: String,

        /// One-based result index when target is search text.
        #[arg(short, long, default_value_t = 1)]
        select: usize,

        /// Player executable, for example potplayer, PotPlayerMini64.exe, mpv, or vlc.
        #[arg(short, long, env = "EMBYCLI_PLAYER")]
        player: Option<String>,

        /// Print the streaming URL instead of launching a player.
        #[arg(long)]
        print_url: bool,
    },

    /// Print shell completion script.
    Completions {
        /// Shell to generate completions for.
        shell: CompletionShell,
    },
}

#[derive(Clone, Debug, ValueEnum)]
enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

impl From<CompletionShell> for Shell {
    fn from(value: CompletionShell) -> Self {
        match value {
            CompletionShell::Bash => Shell::Bash,
            CompletionShell::Elvish => Shell::Elvish,
            CompletionShell::Fish => Shell::Fish,
            CompletionShell::PowerShell => Shell::PowerShell,
            CompletionShell::Zsh => Shell::Zsh,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    server: String,
    user_id: String,
    username: String,
    access_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthResponse {
    user: User,
    access_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct User {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ItemsResponse {
    items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Item {
    id: String,
    name: String,
    #[serde(rename = "Type")]
    item_type: String,
    production_year: Option<i32>,
    index_number: Option<i32>,
    parent_index_number: Option<i32>,
    series_name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login {
            server,
            username,
            password,
        } => login(server, username, password).await,
        Commands::Whoami => whoami(),
        Commands::Search { query, limit } => search(query, limit).await,
        Commands::Play {
            target,
            select,
            player,
            print_url,
        } => play(target, select, player, print_url).await,
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            let shell: Shell = shell.into();
            generate(shell, &mut cmd, name, &mut io::stdout());
            Ok(())
        }
    }
}

async fn login(server: String, username: String, password: Option<String>) -> Result<()> {
    let server = normalize_server_url(&server);
    let password = match password {
        Some(password) => password,
        None => rpassword::prompt_password("Password: ")?,
    };

    let client = Client::new();
    let response = client
        .post(api_url(&server, "/Users/AuthenticateByName"))
        .header("Authorization", authorization_header(None))
        .json(&serde_json::json!({
            "Username": username,
            "Pw": password,
        }))
        .send()
        .await
        .context("failed to send login request")?;

    if response.status() == StatusCode::UNAUTHORIZED {
        bail!("login failed: invalid username or password");
    }

    let response = response
        .error_for_status()
        .context("login request failed")?
        .json::<AuthResponse>()
        .await
        .context("failed to parse login response")?;

    let config = Config {
        server,
        user_id: response.user.id,
        username: response.user.name,
        access_token: response.access_token,
    };
    save_config(&config)?;

    println!("Logged in as {} at {}", config.username, config.server);
    println!("Saved credentials to {}", config_path()?.display());
    Ok(())
}

fn whoami() -> Result<()> {
    let config = load_config()?;
    println!("Server: {}", config.server);
    println!("User:   {} ({})", config.username, config.user_id);
    println!("Config: {}", config_path()?.display());
    Ok(())
}

async fn search(query: String, limit: u32) -> Result<()> {
    let config = load_config()?;
    let items = search_items(&config, &query, limit).await?;
    print_items(&items);
    Ok(())
}

async fn play(
    target: String,
    select: usize,
    player: Option<String>,
    print_url: bool,
) -> Result<()> {
    if select == 0 {
        bail!("--select is one-based and must be greater than zero");
    }

    let config = load_config()?;
    let item = if looks_like_emby_id(&target) {
        match get_item(&config, &target).await {
            Ok(item) => item,
            Err(error) if is_not_found(&error) => {
                select_search_item(&config, &target, select).await?
            }
            Err(error) => return Err(error),
        }
    } else {
        select_search_item(&config, &target, select).await?
    };

    let stream_url = stream_url(&config, &item.id);
    if print_url {
        println!("{stream_url}");
        return Ok(());
    }

    let player = player.unwrap_or_else(default_player);
    println!("Playing: {}", display_title(&item));
    Command::new(&player)
        .arg(&stream_url)
        .spawn()
        .with_context(|| format!("failed to launch player {player:?}"))?;

    Ok(())
}

async fn select_search_item(config: &Config, query: &str, select: usize) -> Result<Item> {
    let limit = select.max(10) as u32;
    let items = search_items(config, query, limit).await?;
    if items.is_empty() {
        bail!("no search results for {query:?}");
    }
    print_items(&items);
    items
        .into_iter()
        .nth(select - 1)
        .ok_or_else(|| anyhow!("--select {select} is outside the result list"))
}

async fn search_items(config: &Config, query: &str, limit: u32) -> Result<Vec<Item>> {
    let client = Client::new();
    let item_types = "Movie,Series,Episode,MusicVideo,Video";
    let url = format!(
        "{}?Recursive=true&SearchTerm={}&IncludeItemTypes={item_types}&Fields=ProductionYear,SeriesName,ParentIndexNumber,IndexNumber&Limit={limit}",
        api_url(&config.server, &format!("/Users/{}/Items", config.user_id)),
        urlencoding::encode(query),
    );

    let response = client
        .get(url)
        .header("Authorization", authorization_header(Some(config)))
        .header("X-Emby-Token", &config.access_token)
        .send()
        .await
        .context("failed to send search request")?
        .error_for_status()
        .context("search request failed")?
        .json::<ItemsResponse>()
        .await
        .context("failed to parse search response")?;

    Ok(response.items)
}

async fn get_item(config: &Config, id: &str) -> Result<Item> {
    let client = Client::new();
    client
        .get(api_url(
            &config.server,
            &format!(
                "/Users/{}/Items/{id}?Fields=ProductionYear,SeriesName,ParentIndexNumber,IndexNumber",
                config.user_id
            ),
        ))
        .header("Authorization", authorization_header(Some(config)))
        .header("X-Emby-Token", &config.access_token)
        .send()
        .await
        .context("failed to send item request")?
        .error_for_status()
        .context("item request failed")?
        .json::<Item>()
        .await
        .context("failed to parse item response")
}

fn is_not_found(error: &anyhow::Error) -> bool {
    error
        .chain()
        .filter_map(|cause| cause.downcast_ref::<reqwest::Error>())
        .any(|error| error.status() == Some(StatusCode::NOT_FOUND))
}

fn print_items(items: &[Item]) {
    if items.is_empty() {
        println!("No results");
        return;
    }

    for (index, item) in items.iter().enumerate() {
        println!(
            "{:>2}. {:<10} {}  [{}]",
            index + 1,
            item.item_type,
            display_title(item),
            item.id
        );
    }
}

fn display_title(item: &Item) -> String {
    match item.item_type.as_str() {
        "Episode" => {
            let prefix = match (
                &item.series_name,
                item.parent_index_number,
                item.index_number,
            ) {
                (Some(series), Some(season), Some(episode)) => {
                    format!("{series} S{season:02}E{episode:02} - ")
                }
                (Some(series), _, _) => format!("{series} - "),
                _ => String::new(),
            };
            format!("{prefix}{}", item.name)
        }
        _ => match item.production_year {
            Some(year) => format!("{} ({year})", item.name),
            None => item.name.clone(),
        },
    }
}

fn stream_url(config: &Config, item_id: &str) -> String {
    format!(
        "{}?Static=true&api_key={}",
        api_url(&config.server, &format!("/Videos/{item_id}/stream")),
        urlencoding::encode(&config.access_token)
    )
}

fn authorization_header(config: Option<&Config>) -> String {
    let user_id = config.map(|config| config.user_id.as_str()).unwrap_or("");
    format!(
        "Emby UserId=\"{user_id}\", Client=\"{CLIENT_NAME}\", Device=\"CLI\", DeviceId=\"{CLIENT_NAME}\", Version=\"{CLIENT_VERSION}\"",
    )
}

fn normalize_server_url(server: &str) -> String {
    server.trim().trim_end_matches('/').to_string()
}

fn api_url(server: &str, path: &str) -> String {
    format!("{}/emby{}", server.trim_end_matches('/'), path)
}

fn looks_like_emby_id(target: &str) -> bool {
    !target.is_empty() && target.chars().all(|c| c.is_ascii_digit())
        || target.len() >= 24 && target.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::looks_like_emby_id;

    #[test]
    fn numeric_item_ids_are_direct_ids() {
        assert!(looks_like_emby_id("45644"));
    }

    #[test]
    fn long_hex_item_ids_are_direct_ids() {
        assert!(looks_like_emby_id("0123456789abcdef01234567"));
    }

    #[test]
    fn search_text_is_not_a_direct_id() {
        assert!(!looks_like_emby_id(""));
        assert!(!looks_like_emby_id("matrix"));
        assert!(!looks_like_emby_id("matrix 1999"));
    }
}

fn default_player() -> String {
    if cfg!(target_os = "windows") {
        "PotPlayerMini64.exe".to_string()
    } else {
        "mpv".to_string()
    }
}

fn config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("could not determine the user config directory"))?
        .join("embycli");
    Ok(config_dir.join("config.json"))
}

fn load_config() -> Result<Config> {
    let path = config_path()?;
    let config = fs::read_to_string(&path)
        .with_context(|| format!("no saved login found at {}", path.display()))?;
    serde_json::from_str(&config).with_context(|| format!("invalid config at {}", path.display()))
}

fn save_config(config: &Config) -> Result<()> {
    let path = config_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("invalid config path {}", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create config directory {}", parent.display()))?;
    let contents = serde_json::to_string_pretty(config)?;
    let mut file =
        fs::File::create(&path).with_context(|| format!("failed to write {}", path.display()))?;
    file.write_all(contents.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}
