use std::fs;

use zed_extension_api::{
    self as zed, serde_json::{self, Value}, settings::LspSettings, LanguageServerId,
    LanguageServerInstallationStatus, Result,
};

const BINARY_NAME: &str = "iris";
const GITHUB_REPO: &str = "purefunctor/purescript-iris";
/// Downloads extract to `iris-<tag>/` in the extension's work dir.
const VERSION_DIR_PREFIX: &str = "iris-";

/// Matches what the VS Code extension spawns: `iris lsp --stdio`.
const LSP_SUBCOMMAND: &str = "lsp";
const STDIO_ARG: &str = "--stdio";

/// The section Iris requests. ZED LOOKS IT UP AS ONE LITERAL KEY, NOT A DOTTED PATH.
const CONFIG_SECTION: &str = "iris.server";

struct PurescriptIrisExtension {
    cached_binary_path: Option<String>,
}

/// Release asset names, e.g. `iris-x86_64-pc-windows-msvc.zip`, extracting to a dir of the same stem.
fn release_target() -> Result<(&'static str, zed::DownloadedFileType)> {
    use zed::{Architecture, DownloadedFileType, Os};
    match zed::current_platform() {
        (Os::Mac, _) => Ok(("universal-apple-darwin", DownloadedFileType::GzipTar)),
        (Os::Linux, Architecture::X8664) => {
            Ok(("x86_64-unknown-linux-gnu", DownloadedFileType::GzipTar))
        }
        (Os::Windows, Architecture::X8664) => {
            Ok(("x86_64-pc-windows-msvc", DownloadedFileType::Zip))
        }
        (os, arch) => Err(format!(
            "Iris publishes no release for {os:?} {arch:?}. Install it yourself and set \
             lsp.iris.binary.path in your Zed settings."
        )),
    }
}

fn binary_path_in(version_dir: &str, target: &str) -> String {
    let exe = match zed::current_platform().0 {
        zed::Os::Windows => ".exe",
        _ => "",
    };
    format!("{version_dir}/{BINARY_NAME}-{target}/{BINARY_NAME}{exe}")
}

/// Most recently modified previous download, for when GitHub can't be reached.
fn existing_download(target: &str) -> Option<String> {
    fs::read_dir(".")
        .ok()?
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(VERSION_DIR_PREFIX))
        .filter_map(|entry| {
            let path = binary_path_in(&entry.file_name().to_string_lossy(), target);
            let modified = fs::metadata(&path).ok()?.modified().ok()?;
            Some((modified, path))
        })
        .max()
        .map(|(_, path)| path)
}

impl PurescriptIrisExtension {
    fn downloaded_binary_path(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        let (target, file_type) = release_target()?;

        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = match zed::latest_github_release(
            GITHUB_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        ) {
            Ok(release) => release,
            Err(err) => {
                let path = existing_download(target).ok_or(err)?;
                self.cached_binary_path = Some(path.clone());
                return Ok(path);
            }
        };

        let asset_name = format!(
            "{BINARY_NAME}-{target}.{}",
            match file_type {
                zed::DownloadedFileType::Zip => "zip",
                _ => "tar.gz",
            }
        );
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("Iris {} has no asset {asset_name}", release.version))?;

        let version_dir = format!("{VERSION_DIR_PREFIX}{}", release.version);
        let binary_path = binary_path_in(&version_dir, target);

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &LanguageServerInstallationStatus::Downloading,
            );
            zed::download_file(&asset.download_url, &version_dir, file_type)
                .map_err(|err| format!("failed to download Iris {}: {err}", release.version))?;
            zed::make_file_executable(&binary_path)?;

            for entry in fs::read_dir(".").into_iter().flatten().flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.starts_with(VERSION_DIR_PREFIX) && name != version_dir {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::None,
        );
        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for PurescriptIrisExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|settings| settings.binary);

        let command = match binary.as_ref().and_then(|binary| binary.path.clone()) {
            Some(path) => path,
            None => self
                .downloaded_binary_path(language_server_id)
                .inspect_err(|err| {
                    zed::set_language_server_installation_status(
                        language_server_id,
                        &LanguageServerInstallationStatus::Failed(err.clone()),
                    )
                })?,
        };

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

        // Iris shells out for source discovery, so `binary.env` overlays the worktree's
        // shell env rather than replacing it.
        let overrides = binary.and_then(|binary| binary.env).unwrap_or_default();
        // WINDOWS ENV NAMES ARE CASE-INSENSITIVE: AN OVERRIDE OF `PATH` MUST DROP `Path`,
        // OR ZED'S HASHMAP KEEPS BOTH AND THE CHILD GETS WHICHEVER IT ITERATES LAST.
        let same_name = |a: &str, b: &str| match zed::current_platform().0 {
            zed::Os::Windows => a.eq_ignore_ascii_case(b),
            _ => a == b,
        };
        let mut env = worktree.shell_env();
        env.retain(|(name, _)| !overrides.keys().any(|key| same_name(name, key)));
        env.extend(overrides);

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
