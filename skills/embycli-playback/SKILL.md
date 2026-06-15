---
name: embycli-playback
description: Uses the local embycli command-line Emby client to search a user's Emby server and play requested movies, series episodes, music videos, videos, or direct item IDs in an external player. Use when the user asks an agent to find, open, stream, watch, play, continue, mark watched/unwatched, or choose Emby media through embycli.
---

# EmbyCLI Playback

Use `embycli` as the playback tool for user-facing Emby media requests. Prefer non-interactive commands so the agent can finish the action without waiting at prompts.

## Command runner

If `embycli` is installed on `PATH`, run:

```bash
embycli ...
```

If working inside the embycli source repo and the binary is not installed, run:

```bash
cargo run -- ...
```

Honor active local command wrappers. If repository instructions require `rtk`, prefix shell commands with `rtk`.

## Playback workflow

1. Verify the CLI and active account:

```bash
embycli --help
embycli whoami
```

If there is no saved account, ask the user for server URL, username, and optional local account name. Do not ask for passwords in chat. Prefer local hidden input:

```bash
embycli login <server-url> --username <username> --name <account-name>
```

2. Search first when the user gives a title, partial title, or vague request:

```bash
embycli search "matrix" --limit 10
```

Use the numbered result list to choose the target. If one result is clearly intended, proceed. If several results are plausible, ask a concise clarification.

3. Play with a stable one-based selection:

```bash
embycli play "matrix" --select 1
```

For a direct Emby item ID:

```bash
embycli play 123456
```

4. For series, map the user's intent to episode flags:

```bash
embycli play "show name" --select 1 --next-unwatched
embycli play "show name" --select 1 --season 1 --episode 3
```

Use `--next-unwatched` when the user says "continue", "next episode", or asks to play a series without specifying an episode. Use `--choose-episode` only when the user wants interactive selection.

5. Respect requested account or player:

```bash
embycli play "matrix" --account home --player potplayer --select 1
embycli play "matrix" --player "C:/Program Files/DAUM/PotPlayer/PotPlayerMini64.exe" --select 1
```

Saved players:

```bash
embycli players list
embycli players add potplayer --path "C:/Program Files/DAUM/PotPlayer/PotPlayerMini64.exe" --default
embycli players use potplayer
```

## Watched state

Only change watched state when the user asks for it, or when they explicitly want playback to mark the item played:

```bash
embycli play "show name" --select 1 --season 1 --episode 3 --mark-played
embycli watched played "movie title" --select 1
embycli watched unplayed "show name" --season 1 --episode 3
```

## Sensitive output

Avoid `--print-url` unless the user specifically asks for a stream URL or debugging output. Stream URLs include `api_key` and are sensitive.

Do not print or copy `~/.config/embycli/config.json`; it contains Emby access tokens.

## Reference

For concrete commands and troubleshooting steps, read [references/playback-cookbook.md](references/playback-cookbook.md).
