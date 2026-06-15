# embycli

`embycli` is a command-line Emby client written in Rust. It can log in to one or more Emby servers, save account tokens locally, search your media library, choose media interactively, and open stream URLs with external players such as PotPlayer, mpv, or VLC.

## Features

- Log in with Emby username and password.
- Save multiple Emby servers/accounts in a local config file.
- Choose the default account for search and playback.
- Search movies, series, episodes, music videos, and videos.
- Play by search keyword or direct Emby item id.
- Choose search results, seasons, and episodes interactively.
- Continue a series from the next episode not marked as played in Emby.
- Mark episodes or items as played/unplayed in Emby.
- Save multiple local players and choose a default player.
- Launch any local player executable path that accepts a media URL.
- Generate shell completions with `clap_complete`.

## Install

Build from source:

```bash
cargo build --release
```

The binary will be created at:

```bash
target/release/embycli
```

You can copy it into a directory on your `PATH`, or run it directly from the project:

```bash
cargo run -- --help
```

## Login

Log in to your Emby server:

```bash
embycli login http://127.0.0.1:8096 --username alice
```

If `--password` is omitted, `embycli` prompts for the password interactively with hidden input:

```text
Password:
```

You can also pass the password explicitly:

```bash
embycli login http://127.0.0.1:8096 --username alice --password 'your-password'
```

Give an account a shorter local name:

```bash
embycli login http://127.0.0.1:8096 --username alice --name home
embycli login https://emby.example.com --username bob --name remote
```

Interactive input is recommended because command-line arguments may be stored in shell history or visible to other local process inspection tools.

After a successful login, `embycli` saves the server URL, user id, username, and Emby access token to:

```text
~/.config/embycli/config.json
```

## Check Current Login

```bash
embycli whoami
embycli whoami --account home
```

Example output:

```text
Server: http://127.0.0.1:8096
User:   alice (user-id)
Config: /home/alice/.config/embycli/config.json
```

## Accounts

List saved accounts. The `*` marks the default account:

```bash
embycli accounts list
```

Set the default account used by `whoami`, `search`, and `play`:

```bash
embycli accounts use home
```

Use a non-default account for a single command:

```bash
embycli search "matrix" --account remote
embycli play "matrix" --account remote
```

Remove an account:

```bash
embycli accounts remove remote
```

## Search

Search media by keyword:

```bash
embycli search "matrix"
```

Limit the number of results:

```bash
embycli search "matrix" --limit 10
```

Search against a specific account:

```bash
embycli search "matrix" --account home
```

Example output:

```text
 1. Movie      The Matrix (1999)  [item-id]
 2. Movie      The Matrix Reloaded (2003)  [item-id]
```

## Play

Search and choose a result interactively:

```bash
embycli play "matrix"
```

Choose a specific search result by one-based index without prompting:

```bash
embycli play "matrix" --select 2
```

Play a direct Emby item id:

```bash
embycli play item-id
```

Use a specific player:

```bash
embycli play "matrix" --player mpv
embycli play "matrix" --player vlc
embycli play "matrix" --player PotPlayerMini64.exe
```

Use a saved player by name:

```bash
embycli play "matrix" --player potplayer
```

Set a default player with an environment variable for one shell session:

```bash
export EMBYCLI_PLAYER=mpv
embycli play "matrix"
```

On Windows, for PotPlayer you may need to pass the full executable path if it is not on `PATH`:

```powershell
embycli play "matrix" --player "C:\Program Files\DAUM\PotPlayer\PotPlayerMini64.exe"
```

When the selected result is a series, `embycli` prompts you to choose the season and episode. You can also pass them directly:

```bash
embycli play "show name" --season 1 --episode 3
```

By default, when the selected result is a series and you do not pass `--season` or `--episode`, `embycli` plays the first episode that is not marked as played in Emby:

```bash
embycli play "show name"
embycli play "show name" --next-unwatched
```

Force manual season and episode selection:

```bash
embycli play "show name" --choose-episode
```

Mark the selected item as played after launching the player:

```bash
embycli play "show name" --mark-played
embycli play "show name" --season 1 --episode 3 --mark-played
```

Print the generated stream URL without launching a player:

```bash
embycli play "matrix" --print-url
```

## Watched State

Mark a movie, episode, direct item id, or selected search result as played:

```bash
embycli watched played "matrix"
embycli watched played item-id
```

For series, pass a season and episode or choose interactively:

```bash
embycli watched played "show name" --season 1 --episode 3
embycli watched unplayed "show name" --season 1 --episode 3
```

The watched state is synced to Emby for the selected account. External players launched by `embycli` do not automatically report precise playback position, so this sync tracks played/unplayed item state rather than second-by-second resume progress.

## Players

Save a player command or path:

```bash
embycli players add mpv --path mpv --default
embycli players add vlc --path vlc
```

On Windows, paste a full player path:

```powershell
embycli players add potplayer --path "C:\Program Files\DAUM\PotPlayer\PotPlayerMini64.exe" --default
```

If `--path` is omitted, `embycli` prompts for the path so you can paste it:

```bash
embycli players add potplayer
```

Manage saved players:

```bash
embycli players list
embycli players use mpv
embycli players remove vlc
```

## Shell Completions

Generate completions:

```bash
embycli completions bash
embycli completions zsh
embycli completions fish
embycli completions powershell
embycli completions elvish
```

Example for bash:

```bash
embycli completions bash > ~/.local/share/bash-completion/completions/embycli
```

Example for zsh:

```bash
embycli completions zsh > ~/.zfunc/_embycli
```

Make sure your shell completion directory is loaded by your shell configuration.

## Command Reference

```text
embycli <COMMAND>

Commands:
  login        Log in to an Emby server and save the access token locally
  whoami       Show the active or selected saved server and user
  accounts     Manage saved Emby servers/accounts
  players      Manage saved players and the default player
  watched      Mark items as played or unplayed in Emby
  search       Search movies, series, episodes, music videos, and videos
  play         Play an item by id, or search text and play the selected result
  completions  Print shell completion script
  help         Print this message or the help of the given subcommand(s)
```

### `login`

```text
embycli login [OPTIONS] --username <USERNAME> <SERVER>

Arguments:
  <SERVER>  Emby server base URL, for example http://127.0.0.1:8096

Options:
  -u, --username <USERNAME>  Emby username
  -p, --password <PASSWORD>  Password. Omit this option to enter it interactively
  -n, --name <NAME>          Local account name. Defaults to username@server
```

### `accounts`

```text
embycli accounts <COMMAND>

Commands:
  list    List saved accounts
  use     Set the default account used by search/play/whoami
  remove  Remove a saved account
```

### `players`

```text
embycli players <COMMAND>

Commands:
  list    List saved players
  add     Add or update a saved player
  use     Set the default player
  remove  Remove a saved player
```

### `watched`

```text
embycli watched <COMMAND>

Commands:
  played    Mark an item, search result, or selected series episode as played
  unplayed  Mark an item, search result, or selected series episode as unplayed
```

### `search`

```text
embycli search [OPTIONS] <QUERY>

Arguments:
  <QUERY>  Search text

Options:
  -l, --limit <LIMIT>  Maximum number of results [default: 20]
  -a, --account <ACCOUNT>  Saved account name. Defaults to the configured default account
```

### `play`

```text
embycli play [OPTIONS] <TARGET>

Arguments:
  <TARGET>  Item id or search text

Options:
  -s, --select <SELECT>    One-based result index when target is search text. Omit to choose interactively
  -p, --player <PLAYER>    Player name, executable, or full executable path [env: EMBYCLI_PLAYER=]
  -a, --account <ACCOUNT>  Saved account name. Defaults to the configured default account
      --season <SEASON>    Season number to play when the selected item is a series
      --episode <EPISODE>  Episode number to play when the selected item is a series
      --next-unwatched     Play the next unplayed episode when the selected item is a series
      --choose-episode     Choose season and episode interactively instead of auto-selecting the next unplayed episode
      --mark-played        Mark the selected item as played after launching the player
      --print-url          Print the streaming URL instead of launching a player
```

## Emby API Usage

`embycli` talks to the Emby REST API under:

```text
http[s]://hostname:port/emby/{api_path}
```

Login uses:

```text
POST /emby/Users/AuthenticateByName
```

Search uses:

```text
GET /emby/Users/{UserId}/Items
```

Playback uses a stream URL shaped like:

```text
GET /emby/Videos/{ItemId}/stream?Static=true&api_key={AccessToken}
```

## Security Notes

- `embycli` stores the Emby access token locally in `~/.config/embycli/config.json`.
- The saved config is not encrypted.
- Prefer interactive password input instead of `--password`.
- Protect your local user account and config directory permissions.
- Delete `~/.config/embycli/config.json` to remove the saved login.

## Development

Format, lint, and test:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Build:

```bash
cargo build
```

## Project Structure

```text
src/main.rs      CLI entry point and command dispatch
src/cli.rs       clap command and option definitions
src/commands.rs  command workflows
src/config.rs    config file models, migration, and persistence
src/emby.rs      Emby API client helpers and item formatting
src/player.rs    player resolution and defaults
src/prompt.rs    small interactive prompt helpers
```
