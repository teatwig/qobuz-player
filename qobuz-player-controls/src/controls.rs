use std::time::Duration;

#[derive(Debug)]
pub enum ControlCommand {
    Album { id: String, index: u32 },
    Playlist { id: u32, index: u32, shuffle: bool },
    ArtistTopTracks { artist_id: u32, index: u32 },
    Track { id: u32 },
    SkipToPosition { new_position: u32, force: bool },
    Next,
    Previous,
    PlayPause,
    Play,
    Pause,
    JumpForward,
    JumpBackward,
    Seek { time: Duration },
    SetVolume { volume: f32 },
}

#[derive(Debug, Clone)]
pub struct Controls {
    tx: tokio::sync::mpsc::UnboundedSender<ControlCommand>,
}

impl Controls {
    pub fn new(tx: tokio::sync::mpsc::UnboundedSender<ControlCommand>) -> Self {
        Self { tx }
    }

    pub fn next(&self) {
        self.tx.send(ControlCommand::Next).expect("infailable");
    }

    pub fn previous(&self) {
        self.tx.send(ControlCommand::Previous).expect("infailable");
    }

    pub fn play_pause(&self) {
        self.tx.send(ControlCommand::PlayPause).expect("infailable");
    }

    pub fn play(&self) {
        self.tx.send(ControlCommand::Play).expect("infailable");
    }

    pub fn pause(&self) {
        self.tx.send(ControlCommand::Pause).expect("infailable");
    }

    pub fn play_album(&self, id: &str, index: u32) {
        self.tx
            .send(ControlCommand::Album {
                id: id.to_string(),
                index,
            })
            .expect("infailable");
    }

    pub fn play_playlist(&self, id: u32, index: u32, shuffle: bool) {
        self.tx
            .send(ControlCommand::Playlist { id, index, shuffle })
            .expect("infailable");
    }

    pub fn play_track(&self, id: u32) {
        self.tx
            .send(ControlCommand::Track { id })
            .expect("infailable");
    }

    pub fn play_top_tracks(&self, artist_id: u32, index: u32) {
        self.tx
            .send(ControlCommand::ArtistTopTracks { artist_id, index })
            .expect("infailable");
    }

    pub fn skip_to_position(&self, index: u32, force: bool) {
        self.tx
            .send(ControlCommand::SkipToPosition {
                new_position: index,
                force,
            })
            .expect("infailable");
    }

    pub fn set_volume(&self, volume: f32) {
        self.tx
            .send(ControlCommand::SetVolume { volume })
            .expect("infailable");
    }

    pub fn seek(&self, time: Duration) {
        self.tx
            .send(ControlCommand::Seek { time })
            .expect("infailable");
    }

    pub fn jump_forward(&self) {
        self.tx
            .send(ControlCommand::JumpForward)
            .expect("infailable");
    }

    pub fn jump_backward(&self) {
        self.tx
            .send(ControlCommand::JumpBackward)
            .expect("infailable");
    }
}
