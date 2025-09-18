use std::sync::Arc;

use clap::{Parser, Subcommand};
use dialoguer::{Input, Password};
use qobuz_player_controls::{
    AudioQuality, TracklistReceiver, VolumeReceiver, client::Client,
    notification::NotificationBroadcast, player::Player,
};
use qobuz_player_database::Database;
use qobuz_player_rfid::RfidState;
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

    #[clap(long, default_value_t = 9888)]
    /// Specify port for the web server.
    port: u16,

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
    #[snafu(display("No username found. Set with config or arguments"))]
    UsernameMissing,
    #[snafu(display("No password found. Set with config or arguments"))]
    PasswordMissing,
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

    let database = Arc::new(Database::new().await?);

    tracing_subscriber::fmt()
        .with_max_level(cli.verbosity)
        .with_target(false)
        .compact()
        .init();

    match cli.command {
        Commands::Open => {
            let database_credentials = database.get_credentials().await?;
            let database_configuration = database.get_configuration().await?;
            let tracklist = database.get_tracklist().await.unwrap_or_default();
            let volume = database.get_volume().await.unwrap_or(1.0);

            let username = match cli.username {
                Some(username) => username,
                None => database_credentials
                    .username
                    .ok_or(Error::UsernameMissing)?,
            };

            let password = match cli.password {
                Some(p) => p,
                None => database_credentials
                    .password
                    .ok_or(Error::PasswordMissing)?,
            };

            let max_audio_quality = cli.max_audio_quality.unwrap_or_else(|| {
                database_configuration
                    .max_audio_quality
                    .try_into()
                    .expect("This should always convert")
            });

            let client = Arc::new(Client::new(username, password, max_audio_quality));

            let broadcast = Arc::new(NotificationBroadcast::new());
            let mut player = Player::new(tracklist, client.clone(), volume, broadcast.clone())?;

            let rfid_state = cli.rfid.then(RfidState::default);

            #[cfg(target_os = "linux")]
            if !cli.disable_mpris {
                let position_receiver = player.position();
                let tracklist_receiver = player.tracklist();
                let volume_receiver = player.volume();
                let status_receiver = player.status();
                let controls = player.controls();
                tokio::spawn(async move {
                    if let Err(e) = qobuz_player_mpris::init(
                        position_receiver,
                        tracklist_receiver,
                        volume_receiver,
                        status_receiver,
                        controls,
                    )
                    .await
                    {
                        exit(!cli.disable_tui && !cli.rfid, e.into());
                    }
                });
            }

            if cli.web {
                let position_receiver = player.position();
                let tracklist_receiver = player.tracklist();
                let volume_receiver = player.volume();
                let status_receiver = player.status();
                let controls = player.controls();
                let rfid_state = rfid_state.clone();
                let broadcast = broadcast.clone();
                let client = client.clone();

                tokio::spawn(async move {
                    if let Err(e) = qobuz_player_web::init(
                        controls,
                        position_receiver,
                        tracklist_receiver,
                        volume_receiver,
                        status_receiver,
                        cli.port,
                        cli.web_secret,
                        rfid_state,
                        broadcast,
                        client,
                    )
                    .await
                    {
                        exit(!cli.disable_tui && !cli.rfid, e.into());
                    }
                });
            }

            #[cfg(feature = "gpio")]
            if cli.gpio {
                let status_receiver = player.status();
                tokio::spawn(async {
                    if let Err(e) = qobuz_player_gpio::init(status_receiver).await {
                        exit(!cli.disable_tui && !cli.rfid, e.into());
                    }
                });
            }

            let tracklist_receiver = player.tracklist();
            let volume_receiver = player.volume();
            let database_clone = database.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    store_state_loop(database_clone, tracklist_receiver, volume_receiver).await
                {
                    exit(!cli.disable_tui && !cli.rfid, e);
                }
            });

            if let Some(rfid_state) = rfid_state {
                let tracklist_receiver = player.tracklist();
                let controls = player.controls();
                tokio::spawn(async move {
                    if let Err(e) = qobuz_player_rfid::init(
                        rfid_state,
                        tracklist_receiver,
                        controls,
                        database,
                        broadcast,
                    )
                    .await
                    {
                        exit(!cli.disable_tui && !cli.rfid, e.into());
                    }
                });
            } else if !cli.disable_tui {
                let position_receiver = player.position();
                let tracklist_receiver = player.tracklist();
                let status_receiver = player.status();
                let controls = player.controls();
                let client = client.clone();
                let broadcast = broadcast.clone();
                tokio::spawn(async move {
                    if let Err(e) = qobuz_player_tui::init(
                        client,
                        broadcast,
                        controls,
                        position_receiver,
                        tracklist_receiver,
                        status_receiver,
                    )
                    .await
                    {
                        exit(!cli.disable_tui && !cli.rfid, e.into());
                    };
                });
            };

            player.player_loop().await?;
            Ok(())
        }
        Commands::Config { command } => match command {
            ConfigCommands::Username {} => {
                if let Ok(username) = Input::new()
                    .with_prompt("Enter your username / email")
                    .interact_text()
                {
                    database.set_username(username).await?;

                    println!("Username saved.");
                }
                Ok(())
            }
            ConfigCommands::Password {} => {
                if let Ok(password) = Password::new()
                    .with_prompt("Enter your password (hidden)")
                    .interact()
                {
                    database.set_password(password).await?;

                    println!("Password saved.");
                }
                Ok(())
            }
            ConfigCommands::MaxAudioQuality { quality } => {
                database.set_max_audio_quality(quality).await?;

                println!("Max audio quality saved.");

                Ok(())
            }
        },
    }
}

async fn store_state_loop(
    database: Arc<Database>,
    mut tracklist_receiver: TracklistReceiver,
    mut volume_receiver: VolumeReceiver,
) -> Result<(), Error> {
    loop {
        tokio::select! {
            Ok(_) = volume_receiver.changed() => {
                let volume = *volume_receiver.borrow_and_update();
                database.set_volume(volume).await?;
            }
            Ok(_) = tracklist_receiver.changed() => {
                let tracklist = tracklist_receiver.borrow_and_update().clone();
                database.set_tracklist(&tracklist).await?;
            },
        }
    }
}

fn exit(cli: bool, error: Error) {
    if cli {
        ratatui::restore();
    }

    eprintln!("{error}");
    std::process::exit(1);
}
