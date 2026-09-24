# zed-purescript-alexandrite

> **Disclaimer:**
>
> This extension completely vibe-coded by AI.

A Zed extension that registers [Alexandrite](https://github.com/purefunctor/purescript-alexandrite)
as a language server for PureScript.

Zed has no settings-only way to add a language server
([zed#52653](https://github.com/zed-industries/zed/issues/52653)), so a server that isn't shipped by
an extension needs one — hence this. It contributes nothing but the server registration; the
PureScript language itself (grammar, file associations) comes from the `purescript` extension, which
must stay installed.

## Install

1. `rustup target add wasm32-wasip1` (one-off; Zed builds extensions for that target).
2. In Zed: command palette → `zed: install dev extension` → pick this directory.

Zed rebuilds it on `zed: reload extensions`, and reinstalls on restart.

## Configure

The server is named `alexandrite`. Enable it for PureScript in `settings.json` — or in a project's
`.zed/settings.json`:

```jsonc
{
  "languages": {
    "PureScript": {
      // LIST purescript-language-server FIRST. Zed routes a request to the first server
      // that handles it and does not fall back, so putting pre-1.0 Alexandrite ahead of
      // it means one wedged server takes every PureScript feature down instead of
      // degrading. Use "!purescript-language-server" to run Alexandrite alone.
      "language_servers": ["purescript-language-server", "alexandrite"]
    }
  }
}
```

The binary is resolved from `PATH` (the installer's default location,
`%LOCALAPPDATA%\Alexandrite\bin`, is already on it). Override per-project if needed:

```jsonc
{
  "lsp": {
    "alexandrite": {
      "binary": {
        "path": "C:/path/to/purescript-alexandrite.exe",
        "arguments": ["--diagnostics-on-open", "false"]
      }
    }
  }
}
```

`--stdio` is appended automatically unless `arguments` already contains it, so extra flags can be
passed without repeating it. Alexandrite discovers sources from `spago.lock` by default; passing
`--source-command <cmd>` also disables that integration.

Flags worth knowing about (`purescript-alexandrite --help` for the full list, v0.0.18 here):

| flag | default | notes |
| --- | --- | --- |
| `--diagnostics-on-open <bool>` | `true` | Zed restores every `.purs` tab from the last session at once, so this is one check per restored tab before anything else is served. |
| `--diagnostics-on-save <bool>` | `true` | |
| `--diagnostics-on-change` | off | |
| `--lsp-log <level>` | `info` | |
| `--query-log <level>` / `--checking-log <level>` | `off` | |

`lsp.alexandrite.settings` and `lsp.alexandrite.initialization_options` are forwarded to the server
as workspace configuration and initialization options.

## Known upstream problems

Both are Alexandrite's, not fixable here:

- **Orphaned processes.** It doesn't exit on LSP `shutdown`/`exit`, so one process per Zed window
  accumulates indefinitely. Check with `Get-Process purescript-alexandrite`.
- **One shared log file.** `purescript-alexandrite --log-file` prints the path it uses, which is
  `%TEMP%\purescript-alexandrite.log` for every instance. The first process holds it and the rest log
  nowhere, so the log is usually stale. There's no flag to redirect it.
