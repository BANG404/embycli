use crate::config::Config;

pub fn resolve_player(config: &Config, requested: Option<String>) -> String {
    if let Some(requested) = requested {
        return config
            .player_path(&requested)
            .unwrap_or(requested.as_str())
            .to_string();
    }

    if let Some(default_name) = config.default_player.as_deref()
        && let Some(path) = config.player_path(default_name)
    {
        return path.to_string();
    }

    default_player()
}

fn default_player() -> String {
    if cfg!(target_os = "windows") {
        "PotPlayerMini64.exe".to_string()
    } else {
        "mpv".to_string()
    }
}
