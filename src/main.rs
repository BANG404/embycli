mod cli;
mod commands;
mod config;
mod emby;
mod player;
mod prompt;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login {
            server,
            username,
            password,
            name,
        } => commands::login(server, username, password, name).await,
        Commands::Whoami { account } => commands::whoami(account),
        Commands::Accounts { command } => commands::accounts(command),
        Commands::Players { command } => commands::players(command),
        Commands::Watched { command } => commands::watched(command).await,
        Commands::Search {
            query,
            limit,
            account,
        } => commands::search(query, limit, account).await,
        Commands::Play {
            target,
            select,
            player,
            account,
            season,
            episode,
            next_unwatched,
            choose_episode,
            mark_played,
            print_url,
        } => {
            commands::play(commands::PlayRequest {
                target,
                select,
                requested_player: player,
                account,
                season,
                episode,
                next_unwatched,
                choose_episode,
                mark_played,
                print_url,
            })
            .await
        }
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            let shell: Shell = shell.into();
            generate(shell, &mut cmd, name, &mut std::io::stdout());
            Ok(())
        }
    }
}
