use zed_extension_api::{
    self as zed, serde_json::Value, settings::LspSettings, LanguageServerId, Result,
};

const BINARY_NAME: &str = "purescript-alexandrite";

/// Matches what the VS Code extension spawns: the bare binary in stdio mode, with
/// no subcommand. `alexandrite lsp` also works, but only `--stdio` is what the
/// published extension is tested against.
const STDIO_ARG: &str = "--stdio";

struct PurescriptAlexandriteExtension;

impl zed::Extension for PurescriptAlexandriteExtension {
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
                     lsp.alexandrite.binary.path in your Zed settings."
                )
            })?;

        let mut args = binary
            .as_ref()
            .and_then(|binary| binary.arguments.clone())
            .unwrap_or_default();

        // APPEND `--stdio`, NEVER SUBSTITUTE IT FOR THE USER'S ARGS. Setting `arguments`
        // to pass a flag like `--diagnostics-on-open false` would otherwise drop stdio,
        // leaving a server that never speaks the protocol.
        if !args.iter().any(|arg| arg == STDIO_ARG) {
            args.push(STDIO_ARG.to_string());
        }

        // Alexandrite shells out for source discovery, so it needs a usable env;
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
                .and_then(|settings| settings.settings),
        )
    }
}

zed::register_extension!(PurescriptAlexandriteExtension);
