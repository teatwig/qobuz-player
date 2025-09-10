use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use rodio::Source;
use rodio::{decoder::DecoderBuilder, queue::queue};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

use crate::Result;
use crate::broadcast::Broadcast;

pub struct Sink {
    stream_handle: rodio::OutputStream,
    sink: rodio::Sink,
    sender: Arc<rodio::queue::SourcesQueueInput>,
    broadcast: Arc<Broadcast>,
    current_download: Arc<Mutex<Option<JoinHandle<()>>>>,
    duration_played: Arc<RwLock<Duration>>,
}

impl Sink {
    pub fn new(broadcast: Arc<Broadcast>, volume: f32) -> Result<Self> {
        let mut stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
        stream_handle.log_on_drop(false);
        let (sender, receiver) = queue(true);

        let sink = rodio::Sink::connect_new(stream_handle.mixer());
        sink.append(receiver);
        sink.set_volume(volume);

        Ok(Self {
            sink,
            stream_handle,
            sender,
            broadcast,
            current_download: Default::default(),
            duration_played: Default::default(),
        })
    }

    pub async fn clear(&mut self) -> Result<()> {
        if let Some(handle) = self.current_download.lock().await.take() {
            handle.abort();
        }

        let volume = self.sink.volume();

        let (sender, receiver) = queue(true);
        let sink = rodio::Sink::connect_new(self.stream_handle.mixer());
        sink.append(receiver);
        sink.set_volume(volume);

        self.sink = sink;
        self.sender = sender;

        *self.duration_played.write().await = Default::default();

        Ok(())
    }

    pub fn play(&self) {
        self.sink.play();
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub async fn seek(&self, duration: Duration) -> Result<()> {
        self.sink.try_seek(duration)?;
        *self.duration_played.write().await = Default::default();

        Ok(())
    }

    pub async fn position(&self) -> Duration {
        let position = self.sink.get_pos();
        let duration_played = self.duration_played.read().await;
        position - *duration_played
    }

    pub fn is_playing(&self) -> bool {
        !self.sink.is_paused()
    }

    pub async fn query_track_url(&self, track_url: &str) -> Result<()> {
        if let Some(handle) = self.current_download.lock().await.take() {
            handle.abort();
        }

        let track_url = track_url.to_string();
        let sender = self.sender.clone();
        let broadcast = self.broadcast.clone();
        let duration_played = self.duration_played.clone();

        let handle = tokio::spawn(async move {
            let resp = reqwest::get(&track_url).await.unwrap();
            let cursor = Cursor::new(resp.bytes().await.unwrap());

            let source = DecoderBuilder::new()
                .with_data(cursor)
                .with_seekable(true)
                .build()
                .unwrap();

            let source_duration = source.total_duration();
            let signal = sender.append_with_signal(source);

            broadcast.done_buffering();

            let (tx, rx) = tokio::sync::oneshot::channel();

            tokio::task::spawn_blocking(move || {
                _ = signal.into_iter().next();
                _ = tx.send(());
            });

            tokio::spawn(async move {
                _ = rx.await;
                if let Some(source_duration) = source_duration {
                    *duration_played.write().await += source_duration;
                }
                broadcast.track_finished();
            });
        });

        *self.current_download.lock().await = Some(handle);
        Ok(())
    }

    pub fn set_volume(&self, volume: f32) {
        let volume_pow = volume.clamp(0.0, 1.0).powi(3);
        self.sink.set_volume(volume_pow);
    }
}
