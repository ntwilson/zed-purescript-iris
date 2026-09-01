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
      // Both servers, Alexandrite first. Use "!purescript-language-server" to run
      // Alexandrite alone.
      "language_servers": ["alexandrite", "purescript-language-server"]
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
        "arguments": ["--stdio"]
      }
    }
  }
}
```

`arguments` defaults to `["--stdio"]`, matching what the VS Code extension spawns. Alexandrite
discovers sources from `spago.lock` by default; pass `--source-command <cmd>` through `arguments` to
use something else (which also disables the `spago.lock` integration).

## Not wired up

Alexandrite's `sourceCommand` setting has no equivalent here beyond passing `--source-command`
yourself. Its VS Code extension exposes only `serverPath` and `sourceCommand`, so nothing else is
missing.
