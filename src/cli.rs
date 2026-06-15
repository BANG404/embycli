use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "embycli",
    version,
    about = "Search and play media from one or more Emby servers"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
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

        /// Local account name. Defaults to username@server.
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Show the active or selected saved server and user.
    Whoami {
        /// Saved account name. Defaults to the configured default account.
        #[arg(short, long)]
        account: Option<String>,
    },

    /// Manage saved Emby servers/accounts.
    Accounts {
        #[command(subcommand)]
        command: AccountCommand,
    },

    /// Manage saved players and the default player.
    Players {
        #[command(subcommand)]
        command: PlayerCommand,
    },

    /// Mark items as played or unplayed in Emby.
    Watched {
        #[command(subcommand)]
        command: WatchedCommand,
    },

    /// Search movies, series, episodes, music videos, and videos.
    Search {
        /// Search text.
        query: String,

        /// Maximum number of results.
        #[arg(short, long, default_value_t = 20)]
        limit: u32,

        /// Saved account name. Defaults to the configured default account.
        #[arg(short, long)]
        account: Option<String>,
    },

    /// Play an item by id, or search text and play the selected result.
    Play {
        /// Item id or search text.
        target: String,

        /// One-based result index when target is search text. Omit to choose interactively.
        #[arg(short, long)]
        select: Option<usize>,

        /// Player name, executable, or full executable path.
        #[arg(short, long, env = "EMBYCLI_PLAYER")]
        player: Option<String>,

        /// Saved account name. Defaults to the configured default account.
        #[arg(short, long)]
        account: Option<String>,

        /// Season number to play when the selected item is a series.
        #[arg(long)]
        season: Option<i32>,

        /// Episode number to play when the selected item is a series.
        #[arg(long)]
        episode: Option<i32>,

        /// Play the next unplayed episode when the selected item is a series.
        #[arg(long)]
        next_unwatched: bool,

        /// Choose season and episode interactively instead of auto-selecting the next unplayed episode.
        #[arg(long)]
        choose_episode: bool,

        /// Mark the selected item as played after launching the player.
        #[arg(long)]
        mark_played: bool,

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

#[derive(Debug, Subcommand)]
pub enum AccountCommand {
    /// List saved accounts.
    List,

    /// Set the default account used by search/play/whoami.
    Use {
        /// Saved account name.
        name: String,
    },

    /// Remove a saved account.
    Remove {
        /// Saved account name.
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum PlayerCommand {
    /// List saved players.
    List,

    /// Add or update a saved player.
    Add {
        /// Player name, for example mpv, vlc, or potplayer.
        name: String,

        /// Player executable or full path. Omit to paste it interactively.
        #[arg(short, long)]
        path: Option<String>,

        /// Set this player as the default.
        #[arg(long)]
        default: bool,
    },

    /// Set the default player.
    Use {
        /// Saved player name.
        name: String,
    },

    /// Remove a saved player.
    Remove {
        /// Saved player name.
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum WatchedCommand {
    /// Mark an item, search result, or selected series episode as played.
    Played {
        /// Item id or search text.
        target: String,

        /// One-based result index when target is search text. Omit to choose interactively.
        #[arg(short, long)]
        select: Option<usize>,

        /// Saved account name. Defaults to the configured default account.
        #[arg(short, long)]
        account: Option<String>,

        /// Season number when the selected item is a series.
        #[arg(long)]
        season: Option<i32>,

        /// Episode number when the selected item is a series.
        #[arg(long)]
        episode: Option<i32>,
    },

    /// Mark an item, search result, or selected series episode as unplayed.
    Unplayed {
        /// Item id or search text.
        target: String,

        /// One-based result index when target is search text. Omit to choose interactively.
        #[arg(short, long)]
        select: Option<usize>,

        /// Saved account name. Defaults to the configured default account.
        #[arg(short, long)]
        account: Option<String>,

        /// Season number when the selected item is a series.
        #[arg(long)]
        season: Option<i32>,

        /// Episode number when the selected item is a series.
        #[arg(long)]
        episode: Option<i32>,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

impl From<CompletionShell> for clap_complete::Shell {
    fn from(value: CompletionShell) -> Self {
        match value {
            CompletionShell::Bash => clap_complete::Shell::Bash,
            CompletionShell::Elvish => clap_complete::Shell::Elvish,
            CompletionShell::Fish => clap_complete::Shell::Fish,
            CompletionShell::PowerShell => clap_complete::Shell::PowerShell,
            CompletionShell::Zsh => clap_complete::Shell::Zsh,
        }
    }
}
