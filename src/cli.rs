use crate::config::{Config, ServerConfig};
use crate::server::ServerManager;
use crate::steam::SteamManager;
use anyhow::{Context, Result};
use log::{error, info, warn};
use std::path::Path;

pub async fn install_server(name: &str, dir: Option<&Path>) -> Result<()> {
    info!("Installing CS2 server: {}", name);

    let base_dir = dir.unwrap_or_else(|| Path::new("./servers"));
    let server_dir = base_dir.join(name);

    // Create server directory structure
    std::fs::create_dir_all(&server_dir)
        .with_context(|| format!("Failed to create server directory: {:?}", server_dir))?;

    // Initialize Steam manager and download CS2 server files
    let steam_manager = SteamManager::new()?;
    if let Err(e) = steam_manager.download_cs2_server(&server_dir).await {
        error!("Failed to download CS2 server files: {}", e);
        return Err(e);
    }

    // Generate default server configuration
    let server_config = ServerConfig::default();
    if let Err(e) = server_config.save(&server_dir.join("server.cfg")) {
        error!("Failed to save server configuration: {}", e);
        return Err(e);
    }

    // Save server metadata
    let mut config = Config::load_or_default()?;
    config.add_server(name.to_string(), server_dir.clone())?;
    if let Err(e) = config.save() {
        error!("Failed to save tool configuration: {}", e);
        return Err(e);
    }

    info!("CS2 server '{}' installed successfully at {:?}", name, server_dir);
    println!("CS2 server '{}' installed successfully at {:?}", name, server_dir);
    Ok(())
}

pub async fn start_server(name: &str) -> Result<()> {
    info!("Starting server: {}", name);

    let config = Config::load_or_default()?;
    let server_path = config.get_server_path(name)?;

    let mut server_manager = ServerManager::new(server_path.clone());
    if let Err(e) = server_manager.start().await {
        error!("Failed to start server '{}': {}", name, e);
        return Err(e);
    }

    info!("Server '{}' started successfully", name);
    println!("Server '{}' started successfully", name);
    Ok(())
}

pub async fn stop_server(name: &str) -> Result<()> {
    info!("Stopping server: {}", name);

    let config = Config::load_or_default()?;
    let server_path = config.get_server_path(name)?;

    let mut server_manager = ServerManager::new(server_path.clone());
    if let Err(e) = server_manager.stop().await {
        error!("Failed to stop server '{}': {}", name, e);
        return Err(e);
    }

    info!("Server '{}' stopped successfully", name);
    println!("Server '{}' stopped successfully", name);
    Ok(())
}

pub async fn server_status(name: Option<&str>) -> Result<()> {
    info!("Checking server status");

    let config = Config::load_or_default()?;

    if let Some(name) = name {
        let server_path = config.get_server_path(name)?;
        let mut server_manager = ServerManager::new(server_path.clone());
        match server_manager.get_status().await {
            Ok(status) => {
                info!("Server '{}' status: {:?}", name, status);
                println!("Server '{}': {:?}", name, status);
            }
            Err(e) => {
                error!("Failed to get status for server '{}': {}", name, e);
                return Err(e);
            }
        }
    } else {
        for server_name in config.list_servers() {
            let server_path = config.get_server_path(&server_name)?;
            let mut server_manager = ServerManager::new(server_path.clone());
            match server_manager.get_status().await {
                Ok(status) => {
                    info!("Server '{}' status: {:?}", server_name, status);
                    println!("{}: {:?}", server_name, status);
                }
                Err(e) => {
                    warn!("Failed to get status for server '{}': {}", server_name, e);
                    println!("{}: Error - {}", server_name, e);
                }
            }
        }
    }

    Ok(())
}

pub async fn update_server(name: &str) -> Result<()> {
    info!("Updating server: {}", name);

    let config = Config::load_or_default()?;
    let server_path = config.get_server_path(name)?;

    let steam_manager = SteamManager::new()?;
    if let Err(e) = steam_manager.update_cs2_server(&server_path).await {
        error!("Failed to update server '{}': {}", name, e);
        return Err(e);
    }

    info!("Server '{}' updated successfully", name);
    println!("Server '{}' updated successfully", name);
    Ok(())
}

pub async fn configure_server(name: &str, key: &str, value: &str) -> Result<()> {
    info!("Configuring server '{}' setting '{}' to '{}'", name, key, value);

    let config = Config::load_or_default()?;
    let server_path = config.get_server_path(name)?;

    let mut server_config = match ServerConfig::load(&server_path.join("server.cfg")) {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("Could not load existing config, creating new one: {}", e);
            ServerConfig::default()
        }
    };

    if let Err(e) = server_config.set(key, value) {
        error!("Invalid configuration key '{}': {}", key, e);
        return Err(e);
    }

    if let Err(e) = server_config.save(&server_path.join("server.cfg")) {
        error!("Failed to save server configuration: {}", e);
        return Err(e);
    }

    info!("Configuration updated successfully");
    println!("Configuration updated: {} = {}", key, value);
    Ok(())
}

pub async fn install_map(name: &str, map: &str) -> Result<()> {
    info!("Installing map '{}' for server '{}'", map, name);

    let config = Config::load_or_default()?;
    let server_path = config.get_server_path(name)?;

    // Determine if map is a URL or local path
    if map.starts_with("http://") || map.starts_with("https://") {
        // Download from URL
        info!("Downloading map from URL: {}", map);
        match reqwest::blocking::get(map) {
            Ok(response) => {
                if response.status().is_success() {
                    let map_data = response.bytes()
                        .with_context(|| "Failed to read map data from response")?;

                    // Extract filename from URL or use default
                    let filename = map.split('/').last().unwrap_or("custom_map.bsp");
                    let maps_dir = server_path.join("game").join("csgo").join("maps");

                    std::fs::create_dir_all(&maps_dir)
                        .with_context(|| format!("Failed to create maps directory: {:?}", maps_dir))?;

                    let map_path = maps_dir.join(filename);
                    std::fs::write(&map_path, map_data)
                        .with_context(|| format!("Failed to write map file: {:?}", map_path))?;

                    info!("Map '{}' installed successfully", filename);
                    println!("Map '{}' installed successfully", filename);
                } else {
                    error!("Failed to download map: HTTP {}", response.status());
                    anyhow::bail!("Failed to download map: HTTP {}", response.status());
                }
            }
            Err(e) => {
                error!("Failed to download map: {}", e);
                return Err(e.into());
            }
        }
    } else {
        // Assume local file path
        let source_path = Path::new(map);
        if !source_path.exists() {
            error!("Local map file not found: {:?}", source_path);
            anyhow::bail!("Local map file not found: {:?}", source_path);
        }

        let maps_dir = server_path.join("game").join("csgo").join("maps");
        std::fs::create_dir_all(&maps_dir)
            .with_context(|| format!("Failed to create maps directory: {:?}", maps_dir))?;

        let filename = source_path.file_name()
            .with_context(|| "Invalid map filename")?;
        let dest_path = maps_dir.join(filename);

        std::fs::copy(source_path, &dest_path)
            .with_context(|| format!("Failed to copy map file to {:?}", dest_path))?;

        info!("Map '{}' installed successfully", filename.to_string_lossy());
        println!("Map '{}' installed successfully", filename.to_string_lossy());
    }

    Ok(())
}

pub async fn install_plugin(server_name: &str, plugin: &str) -> Result<()> {
    info!("Installing plugin '{}' for server '{}'", plugin, server_name);

    let config = Config::load_or_default()?;
    let server_path = config.get_server_path(server_name)?;

    // Define known plugins with their download URLs
    let known_plugins = [
        ("sourcemod", "https://sm.alliedmods.net/smdrop/1.11/sourcemod-1.11.0-git6936-linux.tar.gz"),
        ("metamod", "https://mms.alliedmods.net/mmsdrop/1.11/mmsource-1.11.0-git1148-linux.tar.gz"),
        ("steamworks", "https://github.com/KyleSanderson/SteamWorks/releases/download/1.2.3c/package-lin.tgz"),
    ];

    let plugin_url = if let Some((_, url)) = known_plugins.iter().find(|(name, _)| *name == plugin) {
        *url
    } else if plugin.starts_with("http://") || plugin.starts_with("https://") {
        plugin
    } else {
        error!("Unknown plugin '{}' and not a valid URL", plugin);
        anyhow::bail!("Unknown plugin '{}' and not a valid URL. Use 'cs2-server-cli plugin recommended' to see available plugins.", plugin);
    };

    info!("Downloading plugin from: {}", plugin_url);
    match reqwest::blocking::get(plugin_url) {
        Ok(response) => {
            if response.status().is_success() {
                let plugin_data = response.bytes()
                    .with_context(|| "Failed to read plugin data from response")?;

                // Extract to server directory
                let temp_dir = tempfile::tempdir()
                    .with_context(|| "Failed to create temporary directory")?;

                let archive_path = temp_dir.path().join("plugin_archive");
                std::fs::write(&archive_path, plugin_data)
                    .with_context(|| "Failed to write plugin archive")?;

                // For now, just extract to plugins directory
                // TODO: Proper archive extraction
                let plugins_dir = server_path.join("game").join("csgo").join("addons");
                std::fs::create_dir_all(&plugins_dir)
                    .with_context(|| format!("Failed to create plugins directory: {:?}", plugins_dir))?;

                // Simple extraction for tar.gz (basic implementation)
                if plugin_url.ends_with(".tar.gz") || plugin_url.ends_with(".tgz") {
                    // TODO: Implement proper tar.gz extraction
                    warn!("Tar.gz extraction not fully implemented yet");
                    std::fs::copy(&archive_path, plugins_dir.join(format!("{}.tar.gz", plugin)))
                        .with_context(|| "Failed to save plugin archive")?;
                } else {
                    std::fs::copy(&archive_path, plugins_dir.join(plugin))
                        .with_context(|| "Failed to save plugin file")?;
                }

                info!("Plugin '{}' installed successfully", plugin);
                println!("Plugin '{}' installed successfully", plugin);
                println!("Note: You may need to restart the server for the plugin to take effect.");
            } else {
                error!("Failed to download plugin: HTTP {}", response.status());
                anyhow::bail!("Failed to download plugin: HTTP {}", response.status());
            }
        }
        Err(e) => {
            error!("Failed to download plugin: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

pub async fn list_plugins(server_name: &str) -> Result<()> {
    info!("Listing plugins for server '{}'", server_name);

    let config = Config::load_or_default()?;
    let server_path = config.get_server_path(server_name)?;

    let plugins_dir = server_path.join("game").join("csgo").join("addons");

    if !plugins_dir.exists() {
        info!("No plugins directory found for server '{}'", server_name);
        println!("No plugins installed for server '{}'", server_name);
        return Ok(());
    }

    match std::fs::read_dir(&plugins_dir) {
        Ok(entries) => {
            let mut plugins = Vec::new();
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    plugins.push(name.to_string());
                }
            }

            if plugins.is_empty() {
                println!("No plugins installed for server '{}'", server_name);
            } else {
                println!("Plugins for server '{}':", server_name);
                for plugin in plugins {
                    println!("- {}", plugin);
                }
            }
        }
        Err(e) => {
            warn!("Failed to read plugins directory: {}", e);
            println!("Error reading plugins directory: {}", e);
        }
    }

    Ok(())
}

pub async fn remove_plugin(server_name: &str, plugin: &str) -> Result<()> {
    info!("Removing plugin '{}' from server '{}'", plugin, server_name);

    let config = Config::load_or_default()?;
    let server_path = config.get_server_path(server_name)?;

    let plugins_dir = server_path.join("game").join("csgo").join("addons");
    let plugin_path = plugins_dir.join(plugin);

    if !plugin_path.exists() {
        warn!("Plugin '{}' not found in server '{}'", plugin, server_name);
        anyhow::bail!("Plugin '{}' not found", plugin);
    }

    if plugin_path.is_dir() {
        std::fs::remove_dir_all(&plugin_path)
            .with_context(|| format!("Failed to remove plugin directory: {:?}", plugin_path))?;
    } else {
        std::fs::remove_file(&plugin_path)
            .with_context(|| format!("Failed to remove plugin file: {:?}", plugin_path))?;
    }

    info!("Plugin '{}' removed successfully", plugin);
    println!("Plugin '{}' removed successfully", plugin);
    println!("Note: You may need to restart the server for changes to take effect.");

    Ok(())
}

pub async fn show_recommended_plugins() -> Result<()> {
    println!("Recommended CS2 plugins:");
    println!("1. SourceMod - Server administration and plugin framework");
    println!("2. MetaMod - Plugin management system");
    println!("3. SteamWorks - Steam API integration");
    println!("4. DHooks - Dynamic hooks for Source engine");
    println!("5. Accelerator - Performance optimization");

    Ok(())
}

pub async fn list_servers() -> Result<()> {
    let config = Config::load_or_default()?;
    let servers = config.list_servers();

    if servers.is_empty() {
        println!("No servers installed");
    } else {
        println!("Installed servers:");
        for server in servers {
            println!("- {}", server);
        }
    }

    Ok(())
}

pub async fn backup_server(name: &str, backup_name: &str) -> Result<()> {
    info!("Creating backup '{}' for server '{}'", backup_name, name);

    let config = Config::load_or_default()?;
    let server_path = config.get_server_path(name)?;

    let server_manager = ServerManager::new(server_path.clone());
    if let Err(e) = server_manager.create_backup(backup_name).await {
        error!("Failed to create backup: {}", e);
        return Err(e);
    }

    info!("Backup '{}' created successfully", backup_name);
    println!("Backup '{}' created successfully for server '{}'", backup_name, name);
    Ok(())
}

pub async fn restore_server(name: &str, backup_name: &str) -> Result<()> {
    info!("Restoring backup '{}' for server '{}'", backup_name, name);

    let config = Config::load_or_default()?;
    let server_path = config.get_server_path(name)?;

    let server_manager = ServerManager::new(server_path.clone());
    if let Err(e) = server_manager.restore_backup(backup_name).await {
        error!("Failed to restore backup: {}", e);
        return Err(e);
    }

    info!("Backup '{}' restored successfully", backup_name);
    println!("Backup '{}' restored successfully for server '{}'", backup_name, name);
    println!("Note: You may need to restart the server for changes to take effect.");
    Ok(())
}