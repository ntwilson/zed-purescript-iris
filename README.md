# zed-purescript-iris

> **Disclaimer:**
>
> This extension completely vibe-coded by AI.

A Zed extension that registers [Iris](https://github.com/purefunctor/purescript-iris)
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

The server is named `iris`. Enable it for PureScript in `settings.json` — or in a project's
`.zed/settings.json`:

```jsonc
{
  "languages": {
    "PureScript": {
      // LIST purescript-language-server FIRST. Zed routes a request to the first server
      // that handles it and does not fall back, so putting pre-1.0 Iris ahead of
      // it means one wedged server takes every PureScript feature down instead of
      // degrading. Use "!purescript-language-server" to run Iris alone.
      "language_servers": ["purescript-language-server", "iris"]
    }
  }
}
```

The binary is resolved from `PATH` (add the installer's default location, `%LOCALAPPDATA%\Iris\bin`,
if it isn't already there). Override per-project if needed:

```jsonc
{
  "lsp": {
    "iris": {
      "binary": {
        "path": "C:/path/to/iris.exe",
        "arguments": ["lsp", "--stdio", "--lsp-log", "debug"],
        "env": {
          "IRIS_SPAGO": "node_modules\\.bin\\spago.cmd"
        }
      },
      "settings": {
        "diagnostics": { "onOpen": false }
      }
    }
  }
}
```

**When `binary.path` is set, Zed launches it directly and never asks this extension for the
command, so `arguments` must include `lsp --stdio` itself.** Without `path`, the extension prepends
`lsp` and appends `--stdio` unless `arguments` already has them, so only extra flags need listing.
Flags (`iris lsp --help` for the full list): `--lsp-log <level>` (default
`info`), `--query-log <level>` and `--checking-log <level>` (default `off`). Logs go to
`%LOCALAPPDATA%\iris-lang\iris\cache\iris.log`.

If iris is unable to find your spago command, try setting the env var for IRIS_SPAGO like in the example.

`lsp.iris.settings` is served to Iris as its `iris.server` configuration section; see
[Iris's README](https://github.com/purefunctor/purescript-iris#language-server-configuration) for the
fields. Diagnostics default to on-open and on-save; Zed restores every `.purs` tab from the last
session at once, so `onOpen` is one check per restored tab before anything else is served.
`lsp.iris.initialization_options` is forwarded as-is.

## Known upstream problems

Iris's, not fixable here:

- **Orphaned processes.** It doesn't exit on LSP `shutdown`/`exit`, so one process per Zed window
  accumulates indefinitely. Check with `Get-Process iris`.
