use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::fs;
use tokio::process::Command as TokioCommand;

#[derive(Debug, Clone, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
}

pub struct ServerManager {
    server_path: PathBuf,
    process: Option<tokio::process::Child>,
}

impl ServerManager {
    pub fn new(server_path: PathBuf) -> Self {
        Self {
            server_path,
            process: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.process.is_some() {
            anyhow::bail!("Server is already running");
        }

        let executable = self.get_executable_path()?;
        let _config_path = self.server_path.join("server.cfg");

        let mut command = TokioCommand::new(&executable);
        command
            .current_dir(&self.server_path)
            .arg("+exec")
            .arg("server.cfg")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = command.spawn()
            .with_context(|| format!("Failed to start server process: {:?}", executable))?;

        self.process = Some(child);

        // Wait a moment for the server to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.process.take() {
            // Try graceful shutdown first
            if let Ok(_) = child.kill().await {
                // Wait for the process to exit
                let _ = child.wait().await;
            }
        }

        Ok(())
    }

    pub async fn get_status(&mut self) -> Result<ServerStatus> {
        if let Some(ref mut child) = self.process {
            match child.try_wait() {
                Ok(Some(_)) => Ok(ServerStatus::Stopped),
                Ok(None) => Ok(ServerStatus::Running),
                Err(_) => Ok(ServerStatus::Stopped),
            }
        } else {
            // Check if there's a running process by looking for the executable
            let executable = self.get_executable_path()?;
            if executable.exists() {
                // For now, assume stopped if no process reference
                // TODO: Implement process detection by PID or port checking
                Ok(ServerStatus::Stopped)
            } else {
                Ok(ServerStatus::Stopped)
            }
        }
    }

    pub async fn get_player_count(&self) -> Result<Option<u32>> {
        // TODO: Implement player count retrieval via RCON or log parsing
        Ok(None)
    }

    fn get_executable_path(&self) -> Result<PathBuf> {
        let exe_name = if cfg!(target_os = "windows") {
            "cs2.exe"
        } else {
            "cs2"
        };

        let path = self.server_path.join("game").join("bin").join("linuxsteamrt64").join(exe_name);
        if path.exists() {
            Ok(path)
        } else {
            // Try alternative paths
            let alt_path = self.server_path.join("cs2.exe");
            if alt_path.exists() {
                Ok(alt_path)
            } else {
                anyhow::bail!("CS2 executable not found in expected locations");
            }
        }
    }

    pub async fn create_backup(&self, backup_name: &str) -> Result<()> {
        let backup_dir = self.server_path.join("backups").join(backup_name);
        fs::create_dir_all(&backup_dir).await
            .with_context(|| format!("Failed to create backup directory: {:?}", backup_dir))?;

        // Copy configuration files
        let config_files = ["server.cfg", "autoexec.cfg"];
        for file in &config_files {
            let src = self.server_path.join(file);
            if src.exists() {
                let dst = backup_dir.join(file);
                fs::copy(&src, &dst).await
                    .with_context(|| format!("Failed to backup file: {:?}", file))?;
            }
        }

        Ok(())
    }

    pub async fn restore_backup(&self, backup_name: &str) -> Result<()> {
        let backup_dir = self.server_path.join("backups").join(backup_name);

        if !backup_dir.exists() {
            anyhow::bail!("Backup '{}' not found", backup_name);
        }

        // Restore configuration files
        let config_files = ["server.cfg", "autoexec.cfg"];
        for file in &config_files {
            let src = backup_dir.join(file);
            if src.exists() {
                let dst = self.server_path.join(file);
                fs::copy(&src, &dst).await
                    .with_context(|| format!("Failed to restore file: {:?}", file))?;
            }
        }

        Ok(())
    }

    pub fn list_backups(&self) -> Result<Vec<String>> {
        let backup_dir = self.server_path.join("backups");
        if !backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut backups = vec![];
        for entry in std::fs::read_dir(&backup_dir)
            .with_context(|| format!("Failed to read backup directory: {:?}", backup_dir))? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    backups.push(name.to_string());
                }
            }
        }

        Ok(backups)
    }
}