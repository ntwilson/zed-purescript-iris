use zed_extension_api::{
    self as zed, serde_json::{self, Value}, settings::LspSettings, LanguageServerId, Result,
};

const BINARY_NAME: &str = "iris";

/// Matches what the VS Code extension spawns: `iris lsp --stdio`.
const LSP_SUBCOMMAND: &str = "lsp";
const STDIO_ARG: &str = "--stdio";

/// The section Iris requests. ZED LOOKS IT UP AS ONE LITERAL KEY, NOT A DOTTED PATH.
const CONFIG_SECTION: &str = "iris.server";

struct PurescriptIrisExtension;

impl zed::Extension for PurescriptIrisExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|settings| settings.binary);

        let command = binary
            .as_ref()
            .and_then(|binary| binary.path.clone())
            .or_else(|| worktree.which(BINARY_NAME))
            .ok_or_else(|| {
                format!(
                    "{BINARY_NAME} not found on PATH. Install it, or set \
                     lsp.iris.binary.path in your Zed settings."
                )
            })?;

        let mut args = binary
            .as_ref()
            .and_then(|binary| binary.arguments.clone())
            .unwrap_or_default();

        // ADD `lsp` AND `--stdio`, NEVER SUBSTITUTE THEM FOR THE USER'S ARGS. Setting
        // `arguments` to pass a flag like `--lsp-log debug` would otherwise start no server.
        if args.first().map(String::as_str) != Some(LSP_SUBCOMMAND) {
            args.insert(0, LSP_SUBCOMMAND.to_string());
        }
        if !args.iter().any(|arg| arg == STDIO_ARG) {
            args.push(STDIO_ARG.to_string());
        }

        // Iris shells out for source discovery, so it needs a usable env;
        // fall back to the worktree's rather than an empty one.
        let env = binary
            .and_then(|binary| binary.env)
            .map(|env| env.into_iter().collect())
            .unwrap_or_else(|| worktree.shell_env());

        Ok(zed::Command { command, args, env })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<Value>> {
        Ok(
            LspSettings::for_worktree(language_server_id.as_ref(), worktree)
                .ok()
                .and_then(|settings| settings.initialization_options),
        )
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<Value>> {
        Ok(
            LspSettings::for_worktree(language_server_id.as_ref(), worktree)
                .ok()
                .and_then(|settings| settings.settings)
                .map(|settings| serde_json::json!({ CONFIG_SECTION: settings })),
        )
    }
}

zed::register_extension!(PurescriptIrisExtension);
