# EmbyCLI Playback Cookbook

## Common requests

Play a movie:

```bash
embycli search "movie title" --limit 10
embycli play "movie title" --select <number>
```

Continue a show:

```bash
embycli search "show title" --limit 10
embycli play "show title" --select <number> --next-unwatched
```

Play a specific episode:

```bash
embycli search "show title" --limit 10
embycli play "show title" --select <number> --season <season> --episode <episode>
```

Use a non-default account:

```bash
embycli accounts list
embycli play "title" --account <account> --select <number>
```

Use or configure a player:

```bash
embycli players list
embycli players add <name> --path "<player-path>" --default
embycli play "title" --player <name> --select <number>
```

## Selection rules

- Treat search result indexes as one-based.
- Never pass `--select 0`.
- Search first when the target is natural language.
- Use direct `embycli play <item-id>` only when the user gives an Emby ID or previous output clearly identifies the item ID.
- Ask before choosing when search results contain multiple likely versions, remakes, duplicate libraries, or similarly named episodes.

## Login handling

`embycli whoami` verifies that a default account exists.

If there is no login, ask for server URL, username, and optional local account name. Prefer running login without `--password` so the password is typed locally and hidden:

```bash
embycli login <server-url> --username <username> --name <account-name>
```

Do not echo passwords, access tokens, stream URLs, or config JSON.

## Troubleshooting

If playback does not start:

1. Verify the selected account:

```bash
embycli whoami
embycli accounts list
```

2. Verify the player:

```bash
embycli players list
```

3. Try a known player path or command:

```bash
embycli play "title" --select <number> --player mpv
embycli play "title" --select <number> --player vlc
```

4. Use `--print-url` only for explicit debugging because it exposes a token-bearing URL.
