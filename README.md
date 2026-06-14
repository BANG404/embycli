# embycli

`embycli` is a command-line Emby client written in Rust. It can log in to an Emby server, save the login token locally, search your media library, and open stream URLs with external players such as PotPlayer, mpv, or VLC.

## Features

- Log in with Emby username and password.
- Save the Emby access token in a local config file.
- Search movies, series, episodes, music videos, and videos.
- Play by search keyword or direct Emby item id.
- Launch any local player executable that accepts a media URL.
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

Interactive input is recommended because command-line arguments may be stored in shell history or visible to other local process inspection tools.

After a successful login, `embycli` saves the server URL, user id, username, and Emby access token to:

```text
~/.config/embycli/config.json
```

## Check Current Login

```bash
embycli whoami
```

Example output:

```text
Server: http://127.0.0.1:8096
User:   alice (user-id)
Config: /home/alice/.config/embycli/config.json
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

Example output:

```text
 1. Movie      The Matrix (1999)  [item-id]
 2. Movie      The Matrix Reloaded (2003)  [item-id]
```

## Play

Play the first search result:

```bash
embycli play "matrix"
```

Choose a specific search result by one-based index:

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

Set a default player with an environment variable:

```bash
export EMBYCLI_PLAYER=mpv
embycli play "matrix"
```

On Windows, for PotPlayer you may need to pass the full executable path if it is not on `PATH`:

```powershell
embycli play "matrix" --player "C:\Program Files\DAUM\PotPlayer\PotPlayerMini64.exe"
```

Print the generated stream URL without launching a player:

```bash
embycli play "matrix" --print-url
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
  whoami       Show the saved server and user
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
```

### `search`

```text
embycli search [OPTIONS] <QUERY>

Arguments:
  <QUERY>  Search text

Options:
  -l, --limit <LIMIT>  Maximum number of results [default: 20]
```

### `play`

```text
embycli play [OPTIONS] <TARGET>

Arguments:
  <TARGET>  Item id or search text

Options:
  -s, --select <SELECT>  One-based result index when target is search text [default: 1]
  -p, --player <PLAYER>  Player executable [env: EMBYCLI_PLAYER=]
      --print-url        Print the streaming URL instead of launching a player
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
