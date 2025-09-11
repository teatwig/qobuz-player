use std::sync::Arc;

use qobuz_player_controls::notification::Notification;
use qobuz_player_state::State;
use rppal::gpio::Gpio;

const GPIO: u8 = 23;

pub async fn init(state: Arc<State>) {
    let mut receiver = state.broadcast.notify_receiver();

    let mut pin = Gpio::new().unwrap().get(GPIO).unwrap().into_output();
    tracing::info!("Pin claimed");

    loop {
        if let Ok(notification) = receiver.recv().await {
            match notification {
                Notification::Quit => {
                    return;
                }
                Notification::Status { status } => match status {
                    qobuz_player_controls::Status::Paused => {
                        pin.set_low();
                        tracing::info!("Gpio low");
                    }
                    qobuz_player_controls::Status::Playing
                    | qobuz_player_controls::Status::Buffering => {
                        pin.set_high();
                        tracing::info!("Gpio high");
                    }
                },
                Notification::CurrentTrackList { tracklist: _ } => {}
                Notification::Message { message: _ } => {}
                Notification::Volume { volume: _ } => {}
                Notification::Play(_) => {}
            }
        }
    }
}
