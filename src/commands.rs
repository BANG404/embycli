use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use reqwest::StatusCode;

use crate::{
    cli::{AccountCommand, PlayerCommand, WatchedCommand},
    config::{self, Account, PlayerConfig},
    emby::{self, Item},
    player, prompt,
};

pub async fn login(
    server: String,
    username: String,
    password: Option<String>,
    name: Option<String>,
) -> Result<()> {
    let server = emby::normalize_server_url(&server);
    let password = match password {
        Some(password) => password,
        None => rpassword::prompt_password("Password: ")?,
    };

    let response = match emby::authenticate(&server, username.clone(), password).await {
        Ok(response) => response,
        Err(error) if is_unauthorized(&error) => {
            bail!("login failed: invalid username or password");
        }
        Err(error) => return Err(error),
    };

    let account = Account {
        name: name.unwrap_or_else(|| config::account_name(&response.user.name, &server)),
        server,
        user_id: response.user.id,
        username: response.user.name,
        access_token: response.access_token,
    };

    let mut config = config::load_config_or_empty()?;
    config.upsert_account(account.clone());
    config.default_account = Some(account.name.clone());
    config::save_config(&config)?;

    println!(
        "Logged in as {} at {} ({})",
        account.username, account.server, account.name
    );
    println!("Saved config to {}", config::config_path()?.display());
    Ok(())
}

pub fn whoami(account: Option<String>) -> Result<()> {
    let config = config::load_config()?;
    let account = config.account(account.as_deref())?;
    println!("Account: {}", account.name);
    println!("Server:  {}", account.server);
    println!("User:    {} ({})", account.username, account.user_id);
    println!("Config:  {}", config::config_path()?.display());
    Ok(())
}

pub fn accounts(command: AccountCommand) -> Result<()> {
    let mut config = config::load_config_or_empty()?;
    match command {
        AccountCommand::List => {
            if config.accounts.is_empty() {
                println!("No saved accounts");
                return Ok(());
            }
            for account in &config.accounts {
                let marker = if config.default_account.as_deref() == Some(&account.name) {
                    "*"
                } else {
                    " "
                };
                println!(
                    "{marker} {:<24} {} ({})",
                    account.name, account.server, account.username
                );
            }
            Ok(())
        }
        AccountCommand::Use { name } => {
            config.account(Some(&name))?;
            config.default_account = Some(name.clone());
            config::save_config(&config)?;
            println!("Default account: {name}");
            Ok(())
        }
        AccountCommand::Remove { name } => {
            let account = config.remove_account(&name)?;
            config::save_config(&config)?;
            println!("Removed account: {}", account.name);
            Ok(())
        }
    }
}

pub fn players(command: PlayerCommand) -> Result<()> {
    let mut config = config::load_config_or_empty()?;
    match command {
        PlayerCommand::List => {
            if config.players.is_empty() {
                println!("No saved players");
                return Ok(());
            }
            for saved in &config.players {
                let marker = if config.default_player.as_deref() == Some(&saved.name) {
                    "*"
                } else {
                    " "
                };
                println!("{marker} {:<16} {}", saved.name, saved.path);
            }
            Ok(())
        }
        PlayerCommand::Add {
            name,
            path,
            default,
        } => {
            let path = match path {
                Some(path) => path,
                None => prompt::text("Player path or command: ")?,
            };
            config.upsert_player(
                PlayerConfig {
                    name: name.clone(),
                    path,
                },
                default,
            );
            config::save_config(&config)?;
            if config.default_player.as_deref() == Some(&name) {
                println!("Saved default player: {name}");
            } else {
                println!("Saved player: {name}");
            }
            Ok(())
        }
        PlayerCommand::Use { name } => {
            if config.player_path(&name).is_none() {
                bail!("saved player {name:?} was not found");
            }
            config.default_player = Some(name.clone());
            config::save_config(&config)?;
            println!("Default player: {name}");
            Ok(())
        }
        PlayerCommand::Remove { name } => {
            let removed = config.remove_player(&name)?;
            config::save_config(&config)?;
            println!("Removed player: {}", removed.name);
            Ok(())
        }
    }
}

pub async fn search(query: String, limit: u32, account: Option<String>) -> Result<()> {
    let config = config::load_config()?;
    let account = config.account(account.as_deref())?;
    let items = emby::search_items(account, &query, limit).await?;
    print_items(&items);
    Ok(())
}

pub struct PlayRequest {
    pub target: String,
    pub select: Option<usize>,
    pub requested_player: Option<String>,
    pub account: Option<String>,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub next_unwatched: bool,
    pub choose_episode: bool,
    pub mark_played: bool,
    pub print_url: bool,
}

pub async fn play(request: PlayRequest) -> Result<()> {
    if request.select == Some(0) {
        bail!("--select is one-based and must be greater than zero");
    }

    let config = config::load_config()?;
    let account = config.account(request.account.as_deref())?;
    let selected = resolve_target(account, &request.target, request.select).await?;
    let item = resolve_playable_item(
        account,
        selected,
        request.season,
        request.episode,
        EpisodeSelection::from_flags(request.next_unwatched, request.choose_episode),
    )
    .await?;
    let stream_url = emby::stream_url(account, &item.id);

    if request.print_url {
        println!("{stream_url}");
        return Ok(());
    }

    let player = player::resolve_player(&config, request.requested_player);
    println!("Playing: {}", emby::display_title(&item));
    Command::new(&player)
        .arg(&stream_url)
        .spawn()
        .with_context(|| format!("failed to launch player {player:?}"))?;

    if request.mark_played {
        emby::mark_played(account, &item.id).await?;
        println!("Marked played: {}", emby::display_title(&item));
    }

    Ok(())
}

pub async fn watched(command: WatchedCommand) -> Result<()> {
    match command {
        WatchedCommand::Played {
            target,
            select,
            account,
            season,
            episode,
        } => update_watched_state(target, select, account, season, episode, true).await,
        WatchedCommand::Unplayed {
            target,
            select,
            account,
            season,
            episode,
        } => update_watched_state(target, select, account, season, episode, false).await,
    }
}

async fn update_watched_state(
    target: String,
    select: Option<usize>,
    account: Option<String>,
    season: Option<i32>,
    episode: Option<i32>,
    played: bool,
) -> Result<()> {
    if select == Some(0) {
        bail!("--select is one-based and must be greater than zero");
    }

    let config = config::load_config()?;
    let account = config.account(account.as_deref())?;
    let selected = resolve_target(account, &target, select).await?;
    let item =
        resolve_playable_item(account, selected, season, episode, EpisodeSelection::Manual).await?;

    if played {
        emby::mark_played(account, &item.id).await?;
        println!("Marked played: {}", emby::display_title(&item));
    } else {
        emby::mark_unplayed(account, &item.id).await?;
        println!("Marked unplayed: {}", emby::display_title(&item));
    }

    Ok(())
}

async fn resolve_target(account: &Account, target: &str, select: Option<usize>) -> Result<Item> {
    if emby::looks_like_emby_id(target) {
        match emby::get_item(account, target).await {
            Ok(item) => return Ok(item),
            Err(error) if emby::is_not_found(&error) => {}
            Err(error) => return Err(error),
        }
    }

    select_search_item(account, target, select).await
}

async fn select_search_item(account: &Account, query: &str, select: Option<usize>) -> Result<Item> {
    let limit = select.unwrap_or(20).max(10) as u32;
    let items = emby::search_items(account, query, limit).await?;
    if items.is_empty() {
        bail!("no search results for {query:?}");
    }
    print_items(&items);
    let index = match select {
        Some(select) => select - 1,
        None => prompt::choose_index("Select item: ", items.len())?,
    };
    items
        .into_iter()
        .nth(index)
        .ok_or_else(|| anyhow!("--select {} is outside the result list", index + 1))
}

async fn resolve_playable_item(
    account: &Account,
    item: Item,
    season: Option<i32>,
    episode: Option<i32>,
    selection: EpisodeSelection,
) -> Result<Item> {
    if item.item_type != "Series" {
        return Ok(item);
    }

    if season.is_none() && episode.is_none() && selection == EpisodeSelection::NextUnwatched {
        if let Some(item) = next_unwatched_episode(account, &item).await? {
            println!("Next unwatched: {}", emby::display_title(&item));
            return Ok(item);
        }
        println!("All episodes are marked played; choose an episode.");
    }

    let seasons = emby::seasons(account, &item.id).await?;
    if seasons.is_empty() {
        bail!("series has no seasons: {}", item.name);
    }

    let selected_season = match season {
        Some(season) => seasons
            .iter()
            .find(|item| item.index_number == Some(season))
            .cloned()
            .ok_or_else(|| anyhow!("season {season} was not found"))?,
        None => {
            print_items(&seasons);
            let index = prompt::choose_index("Select season: ", seasons.len())?;
            seasons[index].clone()
        }
    };

    let episodes = emby::episodes(account, &item.id, Some(&selected_season.id)).await?;
    if episodes.is_empty() {
        bail!(
            "season has no episodes: {}",
            emby::display_title(&selected_season)
        );
    }

    match episode {
        Some(episode) => episodes
            .into_iter()
            .find(|item| item.index_number == Some(episode))
            .ok_or_else(|| anyhow!("episode {episode} was not found")),
        None => {
            print_items(&episodes);
            let index = prompt::choose_index("Select episode: ", episodes.len())?;
            Ok(episodes[index].clone())
        }
    }
}

async fn next_unwatched_episode(account: &Account, series: &Item) -> Result<Option<Item>> {
    let mut episodes = emby::episodes(account, &series.id, None).await?;
    episodes.sort_by_key(|item| {
        (
            item.parent_index_number.unwrap_or(i32::MAX),
            item.index_number.unwrap_or(i32::MAX),
            item.name.clone(),
        )
    });
    Ok(episodes.into_iter().find(|item| !emby::is_played(item)))
}

fn print_items(items: &[Item]) {
    if items.is_empty() {
        println!("No results");
        return;
    }

    for (index, item) in items.iter().enumerate() {
        let watched = if emby::is_played(item) { "x" } else { " " };
        println!(
            "{:>2}. [{watched}] {:<10} {}  [{}]",
            index + 1,
            item.item_type,
            emby::display_title(item),
            item.id
        );
    }
}

fn is_unauthorized(error: &anyhow::Error) -> bool {
    error
        .chain()
        .filter_map(|cause| cause.downcast_ref::<reqwest::Error>())
        .any(|error| error.status() == Some(StatusCode::UNAUTHORIZED))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EpisodeSelection {
    NextUnwatched,
    Manual,
}

impl EpisodeSelection {
    fn from_flags(next_unwatched: bool, choose_episode: bool) -> Self {
        match (next_unwatched, choose_episode) {
            (_, true) => Self::Manual,
            (true, false) => Self::NextUnwatched,
            (false, false) => Self::NextUnwatched,
        }
    }
}
