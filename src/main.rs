use clap::{Parser, Subcommand};
use std::path::PathBuf;

use env_logger;

mod cli;
mod config;
mod server;
mod steam;

#[derive(Parser)]
#[command(name = "cs2-server-cli")]
#[command(about = "CLI tool for Counter-Strike 2 server management")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a new CS2 server instance
    Install {
        /// Server instance name
        #[arg(short, long)]
        name: String,
        /// Installation directory
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
    /// Start a server instance
    Start {
        /// Server instance name
        name: String,
    },
    /// Stop a server instance
    Stop {
        /// Server instance name
        name: String,
    },
    /// Get server status
    Status {
        /// Server instance name (optional, shows all if not specified)
        name: Option<String>,
    },
    /// Update server files
    Update {
        /// Server instance name
        name: String,
    },
    /// Configure server settings
    Config {
        /// Server instance name
        name: String,
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Install custom maps
    InstallMap {
        /// Server instance name
        name: String,
        /// Map URL or path
        map: String,
    },
    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        plugin_command: PluginCommands,
    },
    /// List all server instances
    List,
    /// Backup server configuration
    Backup {
        /// Server instance name
        name: String,
        /// Backup name
        backup_name: String,
    },
    /// Restore server configuration
    Restore {
        /// Server instance name
        name: String,
        /// Backup name
        backup_name: String,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// Install a plugin
    Install {
        /// Server instance name
        server_name: String,
        /// Plugin name or URL
        plugin: String,
    },
    /// List installed plugins
    List {
        /// Server instance name
        server_name: String,
    },
    /// Remove a plugin
    Remove {
        /// Server instance name
        server_name: String,
        /// Plugin name
        plugin: String,
    },
    /// Show recommended plugins
    Recommended,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Install { name, dir } => {
            cli::install_server(&name, dir.as_deref()).await?;
        }
        Commands::Start { name } => {
            cli::start_server(&name).await?;
        }
        Commands::Stop { name } => {
            cli::stop_server(&name).await?;
        }
        Commands::Status { name } => {
            cli::server_status(name.as_deref()).await?;
        }
        Commands::Update { name } => {
            cli::update_server(&name).await?;
        }
        Commands::Config { name, key, value } => {
            cli::configure_server(&name, &key, &value).await?;
        }
        Commands::InstallMap { name, map } => {
            cli::install_map(&name, &map).await?;
        }
        Commands::Plugin { plugin_command } => match plugin_command {
            PluginCommands::Install { server_name, plugin } => {
                cli::install_plugin(&server_name, &plugin).await?;
            }
            PluginCommands::List { server_name } => {
                cli::list_plugins(&server_name).await?;
            }
            PluginCommands::Remove { server_name, plugin } => {
                cli::remove_plugin(&server_name, &plugin).await?;
            }
            PluginCommands::Recommended => {
                cli::show_recommended_plugins().await?;
            }
        },
        Commands::List => {
            cli::list_servers().await?;
        }
        Commands::Backup { name, backup_name } => {
            cli::backup_server(&name, &backup_name).await?;
        }
        Commands::Restore { name, backup_name } => {
            cli::restore_server(&name, &backup_name).await?;
        }
    }

    Ok(())
}
