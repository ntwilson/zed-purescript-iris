use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

const BINARY_NAME: &str = "purescript-alexandrite";

/// Matches what the VS Code extension spawns: the bare binary in stdio mode, with
/// no subcommand. `alexandrite lsp` also works, but only `--stdio` is what the
/// published extension is tested against.
const DEFAULT_ARGS: &[&str] = &["--stdio"];

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

        let args = binary
            .as_ref()
            .and_then(|binary| binary.arguments.clone())
            .unwrap_or_else(|| DEFAULT_ARGS.iter().map(|arg| arg.to_string()).collect());

        // Alexandrite shells out for source discovery, so it needs a usable env;
        // fall back to the worktree's rather than an empty one.
        let env = binary
            .and_then(|binary| binary.env)
            .map(|env| env.into_iter().collect())
            .unwrap_or_else(|| worktree.shell_env());

        Ok(zed::Command { command, args, env })
    }
}

zed::register_extension!(PurescriptAlexandriteExtension);
