use anyhow::{Context, Result};
use log::{error, info, warn};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use tokio::process::Command as TokioCommand;
use tokio::signal;

pub struct SteamManager {
    steam_cmd_path: Option<String>,
}

impl SteamManager {
    pub fn new() -> Result<Self> {
        // Try to find steamcmd in common locations
        let steam_cmd_path = Self::find_steamcmd()?;

        Ok(Self {
            steam_cmd_path: Some(steam_cmd_path),
        })
    }

    pub async fn download_cs2_server(&self, install_path: &Path) -> Result<()> {
        info!("Downloading CS2 server files to {:?}", install_path);

        let steam_cmd = self.steam_cmd_path.as_ref()
            .context("SteamCMD not found. Please install SteamCMD and ensure it's in your PATH.")?;

        // CS2 AppID is 730
        let app_id = "730";

        // Create installation script
        let script_content = format!(
            "force_install_dir \"{}\"\n\
             login anonymous\n\
             app_update {} validate\n\
             quit\n",
            install_path.display(),
            app_id
        );

        let script_path = install_path.with_extension("steamscript");
        std::fs::write(&script_path, script_content)
            .with_context(|| format!("Failed to write Steam script: {:?}", script_path))?;

        info!("Running SteamCMD to download CS2 server files");
        // Run SteamCMD
        let mut command = TokioCommand::new(steam_cmd);
        command
            .arg("+runscript")
            .arg(&script_path)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());

        let mut child = command.spawn()
            .with_context(|| "Failed to spawn SteamCMD process")?;

        // Handle Ctrl+C gracefully
        tokio::select! {
            status = child.wait() => {
                match status {
                    Ok(exit_status) if exit_status.success() => {
                        info!("CS2 server files downloaded successfully");
                    }
                    Ok(exit_status) => {
                        error!("SteamCMD failed with exit code: {:?}", exit_status.code());
                        // Clean up script file
                        let _ = std::fs::remove_file(&script_path);
                        anyhow::bail!("SteamCMD failed with exit code: {:?}", exit_status.code());
                    }
                    Err(e) => {
                        error!("Failed to wait for SteamCMD process: {}", e);
                        let _ = std::fs::remove_file(&script_path);
                        return Err(e.into());
                    }
                }
            }
            _ = signal::ctrl_c() => {
                warn!("Received Ctrl+C, terminating SteamCMD process...");
                if let Err(e) = child.kill().await {
                    warn!("Failed to kill SteamCMD process: {}", e);
                }
                let _ = child.wait().await;
                let _ = std::fs::remove_file(&script_path);
                anyhow::bail!("Download interrupted by user");
            }
        }

        // Clean up script file
        if let Err(e) = std::fs::remove_file(&script_path) {
            warn!("Failed to clean up script file: {}", e);
        }

        Ok(())
    }

    pub async fn update_cs2_server(&self, install_path: &Path) -> Result<()> {
        info!("Updating CS2 server files to {:?}", install_path);

        let steam_cmd = self.steam_cmd_path.as_ref()
            .context("SteamCMD not found. Please install SteamCMD and ensure it's in your PATH.")?;

        let app_id = "730";

        let script_content = format!(
            "force_install_dir \"{}\"\n\
             login anonymous\n\
             app_update {} validate\n\
             quit\n",
            install_path.display(),
            app_id
        );

        let script_path = install_path.with_extension("steamscript");
        std::fs::write(&script_path, script_content)
            .with_context(|| format!("Failed to write Steam script: {:?}", script_path))?;

        info!("Running SteamCMD to update CS2 server files");
        let mut command = TokioCommand::new(steam_cmd);
        command
            .arg("+runscript")
            .arg(&script_path)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());

        let mut child = command.spawn()
            .with_context(|| "Failed to spawn SteamCMD process")?;

        // Handle Ctrl+C gracefully
        tokio::select! {
            status = child.wait() => {
                match status {
                    Ok(exit_status) if exit_status.success() => {
                        info!("CS2 server files updated successfully");
                    }
                    Ok(exit_status) => {
                        error!("SteamCMD update failed with exit code: {:?}", exit_status.code());
                        let _ = std::fs::remove_file(&script_path);
                        anyhow::bail!("SteamCMD update failed with exit code: {:?}", exit_status.code());
                    }
                    Err(e) => {
                        error!("Failed to wait for SteamCMD process: {}", e);
                        let _ = std::fs::remove_file(&script_path);
                        return Err(e.into());
                    }
                }
            }
            _ = signal::ctrl_c() => {
                warn!("Received Ctrl+C, terminating SteamCMD process...");
                if let Err(e) = child.kill().await {
                    warn!("Failed to kill SteamCMD process: {}", e);
                }
                let _ = child.wait().await;
                let _ = std::fs::remove_file(&script_path);
                anyhow::bail!("Update interrupted by user");
            }
        }

        let _ = std::fs::remove_file(&script_path);
        Ok(())
    }

    pub async fn download_with_credentials(&self, install_path: &Path, username: &str, password: &str) -> Result<()> {
        println!("Downloading CS2 server files with authentication...");

        let steam_cmd = self.steam_cmd_path.as_ref()
            .context("SteamCMD not found")?;

        let app_id = "730";

        let script_content = format!(
            "force_install_dir \"{}\"\n\
             login {} {}\n\
             app_update {} validate\n\
             quit\n",
            install_path.display(),
            username,
            password,
            app_id
        );

        let script_path = install_path.with_extension("steamscript");
        std::fs::write(&script_path, script_content)
            .with_context(|| format!("Failed to write Steam script: {:?}", script_path))?;

        let mut command = TokioCommand::new(steam_cmd);
        command
            .arg("+runscript")
            .arg(&script_path)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());

        let status = command.status().await
            .with_context(|| "Failed to execute SteamCMD")?;

        if !status.success() {
            anyhow::bail!("SteamCMD failed with exit code: {:?}", status.code());
        }

        let _ = std::fs::remove_file(&script_path);

        println!("CS2 server files downloaded successfully");
        Ok(())
    }

    fn find_steamcmd() -> Result<String> {
        // Check common SteamCMD locations
        let possible_paths = if cfg!(target_os = "windows") {
            vec![
                "C:\\steamcmd\\steamcmd.exe",
                "steamcmd.exe",
            ]
        } else {
            vec![
                "/usr/games/steamcmd",
                "/usr/bin/steamcmd",
                "~/steamcmd/steamcmd.sh",
                "./steamcmd.sh",
                "steamcmd",
            ]
        };

        for path in possible_paths {
            let expanded_path = if path.starts_with("~/") {
                dirs::home_dir()
                    .map(|home| home.join(&path[2..]))
                    .and_then(|p| p.to_str().map(|s| s.to_string()))
            } else {
                Some(path.to_string())
            };

            if let Some(path_str) = expanded_path {
                if std::path::Path::new(&path_str).exists() {
                    return Ok(path_str);
                }

                // Try to run the command to see if it's in PATH
                if Command::new(&path_str).arg("--version").output().is_ok() {
                    return Ok(path_str);
                }
            }
        }

        anyhow::bail!("SteamCMD not found. Please install SteamCMD and ensure it's in your PATH, or specify the full path.");
    }

    pub fn prompt_credentials() -> Result<(String, String)> {
        println!("Steam credentials required for CS2 server installation.");
        println!("Note: Anonymous login may work for basic installation, but authenticated login is recommended.");
        println!();

        print!("Steam Username: ");
        io::stdout().flush()
            .with_context(|| "Failed to flush stdout")?;

        let mut username = String::new();
        io::stdin().read_line(&mut username)
            .with_context(|| "Failed to read username")?;
        let username = username.trim().to_string();

        print!("Steam Password: ");
        io::stdout().flush()
            .with_context(|| "Failed to flush stdout")?;

        let password = rpassword::read_password()
            .with_context(|| "Failed to read password")?;

        println!(); // New line after password input

        if username.is_empty() {
            warn!("Empty username provided, falling back to anonymous login");
            Ok(("anonymous".to_string(), "".to_string()))
        } else {
            info!("Using authenticated Steam login for user: {}", username);
            Ok((username, password))
        }
    }
}