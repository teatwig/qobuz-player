use std::io::Cursor;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use rodio::{decoder::DecoderBuilder, queue::queue};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

use crate::Result;
use crate::broadcast::Broadcast;
use crate::timer::Timer;

pub struct Sink {
    stream_handle: rodio::OutputStream,
    sink: rodio::Sink,
    sender: Arc<rodio::queue::SourcesQueueInput>,
    broadcast: Arc<Broadcast>,
    current_download: Arc<Mutex<Option<JoinHandle<()>>>>,
    position_timer: Arc<RwLock<Timer>>,
    buffering: Arc<AtomicBool>,
}

impl Sink {
    pub fn new(broadcast: Arc<Broadcast>) -> Result<Self> {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
        let (sender, receiver) = queue(true);

        let sink = rodio::Sink::connect_new(stream_handle.mixer());
        sink.append(receiver);
        sink.set_volume(1.0);

        Ok(Self {
            sink,
            stream_handle,
            sender,
            broadcast,
            current_download: Default::default(),
            position_timer: Default::default(),
            buffering: Default::default(),
        })
    }

    pub async fn clear(&mut self) -> Result<()> {
        if let Some(handle) = self.current_download.lock().await.take() {
            handle.abort();
        }

        let (sender, receiver) = queue(true);
        let sink = rodio::Sink::connect_new(self.stream_handle.mixer());
        sink.append(receiver);

        self.sink = sink;
        self.sender = sender;
        self.position_timer.write().await.clear();

        Ok(())
    }

    pub async fn play(&self) {
        self.sink.play();

        if !self.buffering.load(Ordering::Relaxed) {
            self.position_timer.write().await.start();
        }
    }

    pub async fn pause(&self) {
        self.sink.pause();
        self.position_timer.write().await.pause();
    }

    pub fn seek(&self, duration: Duration) -> Result<()> {
        self.sink.try_seek(duration)?;
        Ok(())
    }

    pub async fn position(&self) -> Duration {
        self.position_timer.read().await.elapsed()
    }

    pub fn is_playing(&self) -> bool {
        !self.sink.is_paused()
    }

    pub async fn query_track_url(&self, track_url: &str) -> Result<()> {
        if let Some(handle) = &self.current_download.lock().await.take() {
            handle.abort();
        }

        let track_url = track_url.to_string();
        let sender = self.sender.clone();
        let broadcast = self.broadcast.clone();
        let position_timer = self.position_timer.clone();

        let buffering = self.buffering.clone();
        buffering.store(true, Ordering::Relaxed);

        let handle = tokio::spawn(async move {
            let resp = reqwest::get(&track_url).await.unwrap();
            let cursor = Cursor::new(resp.bytes().await.unwrap());

            let source = DecoderBuilder::new()
                .with_data(cursor)
                .with_seekable(true)
                .build()
                .unwrap();

            buffering.store(false, Ordering::Relaxed);
            position_timer.write().await.start();
            let signal = sender.append_with_signal(source);

            if signal.iter().next().is_some() {
                broadcast.track_finished();
                position_timer.write().await.clear();
            }
        });

        *self.current_download.lock().await = Some(handle);

        self.play().await;
        Ok(())
    }

    pub fn set_volume(&self, volume: f64) {
        let volume_pow = volume.clamp(0.0, 1.0).powi(3);
        self.sink.set_volume(volume_pow as f32);
    }

    pub fn volume(&self) -> f64 {
        self.sink.volume() as f64
    }
}

impl Drop for Sink {
    fn drop(&mut self) {
        self.stream_handle.log_on_drop(false);
        self.sink.clear();
    }
}
