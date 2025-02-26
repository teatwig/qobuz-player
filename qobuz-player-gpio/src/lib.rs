use qobuz_player_controls::notification::Notification;
use rppal::gpio::Gpio;

pub async fn init() {
    tracing::info!("Initiating gpio");
    tokio::spawn(async { receive_notifications().await });
}

const GPIO: u8 = 23;

async fn receive_notifications() {
    let mut broadcast_receiver = qobuz_player_controls::notify_receiver();

    let mut pin = Gpio::new().unwrap().get(GPIO).unwrap().into_output();
    tracing::info!("Pin claimed");

    loop {
        if let Ok(notification) = broadcast_receiver.recv().await {
            match notification {
                Notification::Quit => {
                    return;
                }
                Notification::Status { status } => match status {
                    qobuz_player_controls::State::VoidPending => {
                        pin.set_low();
                        tracing::info!("Gpio low");
                    }
                    qobuz_player_controls::State::Null => {
                        pin.set_low();
                        tracing::info!("Gpio low");
                    }
                    qobuz_player_controls::State::Ready => {
                        pin.set_low();
                        tracing::info!("Gpio low");
                    }
                    qobuz_player_controls::State::Paused => {
                        pin.set_low();
                        tracing::info!("Gpio low");
                    }
                    qobuz_player_controls::State::Playing => {
                        pin.set_high();
                        tracing::info!("Gpio high");
                    }
                },
                Notification::Position { clock: _ } => {}
                Notification::CurrentTrackList { list: _ } => {}
                Notification::Error { error: _ } => {}
                Notification::Volume { volume: _ } => {}
            }
        }
    }
}
