use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default_account: Option<String>,
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub default_player: Option<String>,
    #[serde(default)]
    pub players: Vec<PlayerConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub server: String,
    pub user_id: String,
    pub username: String,
    pub access_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DiskConfig {
    Current(Config),
    Legacy(LegacyConfig),
}

#[derive(Debug, Deserialize)]
struct LegacyConfig {
    server: String,
    user_id: String,
    username: String,
    access_token: String,
}

impl Config {
    pub fn empty() -> Self {
        Self {
            default_account: None,
            accounts: Vec::new(),
            default_player: None,
            players: Vec::new(),
        }
    }

    pub fn upsert_account(&mut self, account: Account) {
        if let Some(saved) = self
            .accounts
            .iter_mut()
            .find(|saved| saved.name == account.name)
        {
            *saved = account;
        } else {
            self.accounts.push(account);
        }

        if self.default_account.is_none() {
            self.default_account = self.accounts.last().map(|account| account.name.clone());
        }
    }

    pub fn account(&self, name: Option<&str>) -> Result<&Account> {
        let name = match name {
            Some(name) => name,
            None => self
                .default_account
                .as_deref()
                .ok_or_else(|| anyhow!("no default account configured; run `embycli login`"))?,
        };

        self.accounts
            .iter()
            .find(|account| account.name == name)
            .ok_or_else(|| anyhow!("saved account {name:?} was not found"))
    }

    pub fn remove_account(&mut self, name: &str) -> Result<Account> {
        let index = self
            .accounts
            .iter()
            .position(|account| account.name == name)
            .ok_or_else(|| anyhow!("saved account {name:?} was not found"))?;
        let account = self.accounts.remove(index);
        if self.default_account.as_deref() == Some(name) {
            self.default_account = self.accounts.first().map(|account| account.name.clone());
        }
        Ok(account)
    }

    pub fn upsert_player(&mut self, player: PlayerConfig, set_default: bool) {
        if let Some(saved) = self
            .players
            .iter_mut()
            .find(|saved| saved.name == player.name)
        {
            *saved = player;
        } else {
            self.players.push(player);
        }

        if set_default || self.default_player.is_none() {
            self.default_player = self.players.last().map(|player| player.name.clone());
        }
    }

    pub fn player_path(&self, name: &str) -> Option<&str> {
        self.players
            .iter()
            .find(|player| player.name == name)
            .map(|player| player.path.as_str())
    }

    pub fn remove_player(&mut self, name: &str) -> Result<PlayerConfig> {
        let index = self
            .players
            .iter()
            .position(|player| player.name == name)
            .ok_or_else(|| anyhow!("saved player {name:?} was not found"))?;
        let player = self.players.remove(index);
        if self.default_player.as_deref() == Some(name) {
            self.default_player = self.players.first().map(|player| player.name.clone());
        }
        Ok(player)
    }
}

impl From<LegacyConfig> for Config {
    fn from(legacy: LegacyConfig) -> Self {
        let name = account_name(&legacy.username, &legacy.server);
        Self {
            default_account: Some(name.clone()),
            accounts: vec![Account {
                name,
                server: legacy.server,
                user_id: legacy.user_id,
                username: legacy.username,
                access_token: legacy.access_token,
            }],
            default_player: None,
            players: Vec::new(),
        }
    }
}

pub fn account_name(username: &str, server: &str) -> String {
    format!(
        "{}@{}",
        username.trim(),
        server.trim().trim_end_matches('/')
    )
}

pub fn config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("could not determine the user config directory"))?
        .join("embycli");
    Ok(config_dir.join("config.json"))
}

pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    load_config_from(&path)
}

pub fn load_config_or_empty() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::empty());
    }
    load_config_from(&path)
}

fn load_config_from(path: &Path) -> Result<Config> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("no saved config found at {}", path.display()))?;
    let config = match serde_json::from_str::<DiskConfig>(&contents)
        .with_context(|| format!("invalid config at {}", path.display()))?
    {
        DiskConfig::Current(config) => config,
        DiskConfig::Legacy(legacy) => legacy.into(),
    };

    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
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
