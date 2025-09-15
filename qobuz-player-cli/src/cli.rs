use std::sync::Arc;

use clap::{Parser, Subcommand};
use dialoguer::{Input, Password};
use qobuz_player_controls::{AudioQuality, TracklistReceiver, client::Client, player::Player};
use qobuz_player_state::{State, database::Database};
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

    let database = Database::new().await;

    tracing_subscriber::fmt()
        .with_max_level(cli.verbosity)
        .with_target(false)
        .compact()
        .init();

    match cli.command {
        Commands::Open => {
            let database_credentials = database.get_credentials().await;
            let database_configuration = database.get_configuration().await;
            let tracklist = database.get_tracklist().await.unwrap_or_default();
            let volume = database.get_volume().await.unwrap_or(1.0);

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

            let client = Arc::new(Client::new(username, password, max_audio_quality));

            let mut player = Player::new(tracklist, client.clone(), volume);
            let broadcast = player.broadcast();

            let state = Arc::new(
                State::new(
                    client,
                    cli.rfid,
                    cli.interface,
                    cli.web_secret,
                    database,
                    broadcast.clone(),
                )
                .await,
            );

            #[cfg(target_os = "linux")]
            if !cli.disable_mpris {
                let state = state.clone();
                let position_receiver = player.position();
                let tracklist_receiver = player.tracklist();
                let volume_receiver = player.volume();
                let status_receiver = player.status();
                tokio::spawn(async move {
                    qobuz_player_mpris::init(
                        state,
                        position_receiver,
                        tracklist_receiver,
                        volume_receiver,
                        status_receiver,
                    )
                    .await;
                });
            }

            if cli.web {
                let state = state.clone();
                let position_receiver = player.position();
                let tracklist_receiver = player.tracklist();
                let volume_receiver = player.volume();
                let status_receiver = player.status();
                tokio::spawn(async move {
                    qobuz_player_web::init(
                        state,
                        position_receiver,
                        tracklist_receiver,
                        volume_receiver,
                        status_receiver,
                    )
                    .await;
                });
            }

            #[cfg(feature = "gpio")]
            if cli.gpio {
                let state = state.clone();
                let status_receiver = player.status();
                tokio::spawn(async {
                    qobuz_player_gpio::init(state, status_receiver).await;
                });
            }

            let state_persist = state.clone();
            let tracklist_receiver = player.tracklist();
            tokio::spawn(async move {
                store_state_loop(state_persist, tracklist_receiver).await;
            });

            if cli.rfid {
                let tracklist_receiver = player.tracklist();
                tokio::spawn(async move {
                    qobuz_player_rfid::init(state, tracklist_receiver).await;
                });
            } else if !cli.disable_tui {
                let position_receiver = player.position();
                let tracklist_receiver = player.tracklist();
                let status_receiver = player.status();
                tokio::spawn(async move {
                    qobuz_player_tui::init(
                        state,
                        position_receiver,
                        tracklist_receiver,
                        status_receiver,
                    )
                    .await;
                });
            };

            player.player_loop().await.unwrap();
            Ok(())
        }
        Commands::Config { command } => match command {
            ConfigCommands::Username {} => {
                if let Ok(username) = Input::new()
                    .with_prompt("Enter your username / email")
                    .interact_text()
                {
                    database.set_username(username).await;

                    println!("Username saved.");
                }
                Ok(())
            }
            ConfigCommands::Password {} => {
                if let Ok(password) = Password::new()
                    .with_prompt("Enter your password (hidden)")
                    .interact()
                {
                    database.set_password(password).await;

                    println!("Password saved.");
                }
                Ok(())
            }
            ConfigCommands::MaxAudioQuality { quality } => {
                database.set_max_audio_quality(quality).await;

                println!("Max audio quality saved.");

                Ok(())
            }
        },
    }
}

async fn store_state_loop(state: Arc<State>, mut tracklist_receiver: TracklistReceiver) {
    loop {
        if tracklist_receiver.changed().await.is_ok() {
            let tracklist = tracklist_receiver.borrow_and_update().clone();
            state.database.set_tracklist(&tracklist).await;
        };
        // tokio::select! {
        //     Ok(_) = tracklist_receiver.changed() => {
        //         let tracklist = tracklist_receiver.borrow_and_update().clone();
        //         state.database.set_tracklist(&tracklist).await;
        //     },
        // }
        // if let Ok(notification) = broadcast_receiver.recv().await {
        //     match notification {
        //         Notification::CurrentTrackList { tracklist } => {
        //             state.database.set_tracklist(&tracklist).await;
        //         }
        //         Notification::Volume { volume } => {
        //             state.database.set_volume(volume).await;
        //         }
        //         _ => (),
        //     }
        // }
    }
}
