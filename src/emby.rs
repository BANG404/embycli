use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::config::Account;

pub const CLIENT_NAME: &str = "embycli";
pub const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthResponse {
    pub user: User,
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct User {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ItemsResponse {
    items: Vec<Item>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
    pub id: String,
    pub name: String,
    #[serde(rename = "Type")]
    pub item_type: String,
    pub production_year: Option<i32>,
    pub index_number: Option<i32>,
    pub parent_index_number: Option<i32>,
    pub series_name: Option<String>,
    pub user_data: Option<UserData>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserData {
    pub played: Option<bool>,
}

pub async fn authenticate(
    server: &str,
    username: String,
    password: String,
) -> Result<AuthResponse> {
    let client = Client::new();
    client
        .post(api_url(server, "/Users/AuthenticateByName"))
        .header("Authorization", authorization_header(None))
        .json(&serde_json::json!({
            "Username": username,
            "Pw": password,
        }))
        .send()
        .await
        .context("failed to send login request")?
        .error_for_status()
        .context("login request failed")?
        .json::<AuthResponse>()
        .await
        .context("failed to parse login response")
}

pub async fn search_items(account: &Account, query: &str, limit: u32) -> Result<Vec<Item>> {
    let item_types = "Movie,Series,Episode,MusicVideo,Video";
    let url = format!(
        "{}?Recursive=true&SearchTerm={}&IncludeItemTypes={item_types}&Fields={}&Limit={limit}",
        api_url(
            &account.server,
            &format!("/Users/{}/Items", account.user_id)
        ),
        urlencoding::encode(query),
        item_fields(),
    );
    get_items(account, url, "search").await
}

pub async fn get_item(account: &Account, id: &str) -> Result<Item> {
    let client = Client::new();
    client
        .get(api_url(
            &account.server,
            &format!(
                "/Users/{}/Items/{id}?Fields={}",
                account.user_id,
                item_fields()
            ),
        ))
        .header("Authorization", authorization_header(Some(account)))
        .header("X-Emby-Token", &account.access_token)
        .send()
        .await
        .context("failed to send item request")?
        .error_for_status()
        .context("item request failed")?
        .json::<Item>()
        .await
        .context("failed to parse item response")
}

pub async fn seasons(account: &Account, series_id: &str) -> Result<Vec<Item>> {
    let url = format!(
        "{}?UserId={}&Fields={}",
        api_url(&account.server, &format!("/Shows/{series_id}/Seasons")),
        urlencoding::encode(&account.user_id),
        item_fields(),
    );
    get_items(account, url, "season list").await
}

pub async fn episodes(
    account: &Account,
    series_id: &str,
    season_id: Option<&str>,
) -> Result<Vec<Item>> {
    let mut url = format!(
        "{}?UserId={}&Fields={}",
        api_url(&account.server, &format!("/Shows/{series_id}/Episodes")),
        urlencoding::encode(&account.user_id),
        item_fields(),
    );
    if let Some(season_id) = season_id {
        url.push_str("&SeasonId=");
        url.push_str(&urlencoding::encode(season_id));
    }
    get_items(account, url, "episode list").await
}

pub async fn mark_played(account: &Account, item_id: &str) -> Result<()> {
    update_played_state(account, item_id, true).await
}

pub async fn mark_unplayed(account: &Account, item_id: &str) -> Result<()> {
    update_played_state(account, item_id, false).await
}

async fn update_played_state(account: &Account, item_id: &str, played: bool) -> Result<()> {
    let client = Client::new();
    let request = if played {
        client.post(api_url(
            &account.server,
            &format!("/Users/{}/PlayedItems/{item_id}", account.user_id),
        ))
    } else {
        client.delete(api_url(
            &account.server,
            &format!("/Users/{}/PlayedItems/{item_id}", account.user_id),
        ))
    };

    request
        .header("Authorization", authorization_header(Some(account)))
        .header("X-Emby-Token", &account.access_token)
        .send()
        .await
        .context("failed to send played-state request")?
        .error_for_status()
        .context("played-state request failed")?;
    Ok(())
}

async fn get_items(account: &Account, url: String, context: &str) -> Result<Vec<Item>> {
    let client = Client::new();
    let response = client
        .get(url)
        .header("Authorization", authorization_header(Some(account)))
        .header("X-Emby-Token", &account.access_token)
        .send()
        .await
        .with_context(|| format!("failed to send {context} request"))?
        .error_for_status()
        .with_context(|| format!("{context} request failed"))?
        .json::<ItemsResponse>()
        .await
        .with_context(|| format!("failed to parse {context} response"))?;
    Ok(response.items)
}

pub fn is_not_found(error: &anyhow::Error) -> bool {
    error
        .chain()
        .filter_map(|cause| cause.downcast_ref::<reqwest::Error>())
        .any(|error| error.status() == Some(StatusCode::NOT_FOUND))
}

pub fn stream_url(account: &Account, item_id: &str) -> String {
    format!(
        "{}?Static=true&api_key={}",
        api_url(&account.server, &format!("/Videos/{item_id}/stream")),
        urlencoding::encode(&account.access_token)
    )
}

pub fn authorization_header(account: Option<&Account>) -> String {
    let user_id = account
        .map(|account| account.user_id.as_str())
        .unwrap_or("");
    format!(
        "Emby UserId=\"{user_id}\", Client=\"{CLIENT_NAME}\", Device=\"CLI\", DeviceId=\"{CLIENT_NAME}\", Version=\"{CLIENT_VERSION}\"",
    )
}

pub fn normalize_server_url(server: &str) -> String {
    server.trim().trim_end_matches('/').to_string()
}

pub fn api_url(server: &str, path: &str) -> String {
    format!("{}/emby{}", server.trim_end_matches('/'), path)
}

fn item_fields() -> &'static str {
    "ProductionYear,SeriesName,ParentIndexNumber,IndexNumber"
}

pub fn is_played(item: &Item) -> bool {
    item.user_data
        .as_ref()
        .and_then(|user_data| user_data.played)
        .unwrap_or(false)
}

pub fn display_title(item: &Item) -> String {
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
        "Season" => match item.index_number {
            Some(season) => format!("Season {season}: {}", item.name),
            None => item.name.clone(),
        },
        _ => match item.production_year {
            Some(year) => format!("{} ({year})", item.name),
            None => item.name.clone(),
        },
    }
}

pub fn looks_like_emby_id(target: &str) -> bool {
    (!target.is_empty() && target.chars().all(|c| c.is_ascii_digit()))
        || (target.len() >= 24 && target.chars().all(|c| c.is_ascii_hexdigit() || c == '-'))
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
