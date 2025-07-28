use std::sync::Arc;

use clap::{Parser, Subcommand};
use dialoguer::{Input, Password};
use qobuz_player_controls::{AudioQuality, notification::Notification};
use qobuz_player_state::State;
use snafu::prelude::*;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Provide a username. (overrides any database value)
    #[clap(short, long)]
    username: Option<String>,

    #[clap(short, long)]
    /// Provide a password. (overrides any database value)
    password: Option<String>,

    #[clap(short, long, default_value_t = false)]
    /// Disable the TUI interface.
    disable_tui: bool,

    #[cfg(target_os = "linux")]
    #[clap(long, default_value_t = false)]
    /// Disable the mpris interface.
    disable_mpris: bool,

    #[clap(short, long)]
    max_audio_quality: Option<AudioQuality>,

    #[clap(short, long)]
    /// Log level
    verbosity: Option<tracing::Level>,

    #[clap(short, long, default_value_t = false)]
    /// Start web server with websocket API and embedded UI.
    web: bool,

    #[clap(long)]
    /// Secret used for web ui auth.
    web_secret: Option<String>,

    #[clap(long, default_value_t = false)]
    /// Enable rfid interface.
    rfid: bool,

    #[cfg(feature = "gpio")]
    #[clap(long, default_value_t = false)]
    /// Enable gpio interface for raspberry pi. Pin 16 (gpio-23) will be high when playing.
    gpio: bool,

    #[clap(long, default_value = "0.0.0.0:9888")]
    /// Specify a different interface and port for the web server to listen on.
    interface: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Open the player
    Open,
    /// Set configuration options
    Config {
        #[clap(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Set username.
    #[clap(value_parser)]
    Username {},
    /// Set password.
    #[clap(value_parser)]
    Password {},
    /// Set max audio quality.
    #[clap(value_parser)]
    MaxAudioQuality {
        #[clap(value_enum)]
        quality: AudioQuality,
    },
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{error}"))]
    ClientError { error: String },
    #[snafu(display("{error}"))]
    PlayerError { error: String },
    #[snafu(display("{error}"))]
    TerminalError { error: String },
}

impl From<qobuz_player_client::Error> for Error {
    fn from(error: qobuz_player_client::Error) -> Self {
        Error::ClientError {
            error: error.to_string(),
        }
    }
}

impl From<qobuz_player_controls::error::Error> for Error {
    fn from(error: qobuz_player_controls::error::Error) -> Self {
        Error::PlayerError {
            error: error.to_string(),
        }
    }
}

pub async fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    let state = Arc::new(State::new(cli.rfid, cli.interface, cli.web_secret).await);

    tracing_subscriber::fmt()
        .with_max_level(cli.verbosity)
        .with_target(false)
        .compact()
        .init();

    match cli.command {
        Commands::Open => {
            let database_credentials = state.database.get_credentials().await;
            let database_configuration = state.database.get_configuration().await;
            let loaded_tracklist = state.database.get_tracklist().await;

            let username = cli.username.unwrap_or_else(|| {
                database_credentials
                    .username
                    .expect("No username. Set with config or arguments")
            });

            let password = cli.password.unwrap_or_else(|| {
                database_credentials
                    .password
                    .expect("No username. Set with config or arguments")
            });

            let max_audio_quality = cli
                .max_audio_quality
                .unwrap_or_else(|| database_configuration.max_audio_quality.try_into().unwrap());

            #[cfg(target_os = "linux")]
            if !cli.disable_mpris {
                tokio::spawn(async {
                    qobuz_player_mpris::init().await;
                });
            }

            if cli.web {
                let state_rfid = state.clone();
                tokio::spawn(async {
                    qobuz_player_web::init(state_rfid).await;
                });
            }

            #[cfg(feature = "gpio")]
            if cli.gpio {
                tokio::spawn(async {
                    qobuz_player_gpio::init().await;
                });
            }

            tokio::spawn(async {
                match qobuz_player_controls::player_loop(
                    qobuz_player_controls::Credentials { username, password },
                    qobuz_player_controls::Configuration { max_audio_quality },
                    loaded_tracklist,
                )
                .await
                {
                    Ok(_) => debug!("player loop exited successfully"),
                    Err(error) => debug!("player loop error {error}"),
                }
            });

            let state_persist = state.clone();
            tokio::spawn(async {
                store_state_loop(state_persist).await;
            });

            if cli.rfid {
                qobuz_player_rfid::init(state.clone()).await;
                qobuz_player_controls::quit().await?
            } else if !cli.disable_tui {
                qobuz_player_tui::init().await;

                debug!("tui exited, quitting");
                qobuz_player_controls::quit().await?
            } else {
                debug!("waiting for ctrlc");
                tokio::signal::ctrl_c()
                    .await
                    .expect("error waiting for ctrlc");
                debug!("ctrlc received, quitting");
                qobuz_player_controls::quit().await?
            };

            Ok(())
        }
        Commands::Config { command } => match command {
            ConfigCommands::Username {} => {
                if let Ok(username) = Input::new()
                    .with_prompt("Enter your username / email")
                    .interact_text()
                {
                    state.database.set_username(username).await;

                    println!("Username saved.");
                }
                Ok(())
            }
            ConfigCommands::Password {} => {
                if let Ok(password) = Password::new()
                    .with_prompt("Enter your password (hidden)")
                    .interact()
                {
                    state.database.set_password(password).await;

                    println!("Password saved.");
                }
                Ok(())
            }
            ConfigCommands::MaxAudioQuality { quality } => {
                state.database.set_max_audio_quality(quality).await;

                println!("Max audio quality saved.");

                Ok(())
            }
        },
    }
}

async fn store_state_loop(state: Arc<State>) {
    let mut broadcast_receiver = qobuz_player_controls::notify_receiver();

    loop {
        if let Ok(Notification::CurrentTrackList { list }) = broadcast_receiver.recv().await {
            state.database.set_tracklist(list).await;
        }
    }
}
