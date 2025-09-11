use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::{decoder::DecoderBuilder, queue::queue};
use tokio::sync::watch::{self, Receiver, Sender};
use tokio::task::JoinHandle;

use crate::Result;

pub struct Sink {
    stream_handle: rodio::OutputStream,
    sink: rodio::Sink,
    sender: Arc<rodio::queue::SourcesQueueInput>,
    current_download: Arc<Mutex<Option<JoinHandle<()>>>>,
    track_finished_tx: Sender<()>,
    done_buffering_tx: Sender<()>,
}

impl Sink {
    pub fn new(volume: f32) -> Result<Self> {
        let mut stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
        stream_handle.log_on_drop(false);
        let (sender, receiver) = queue(true);

        let sink = rodio::Sink::connect_new(stream_handle.mixer());
        sink.append(receiver);
        set_volume(&sink, volume);

        let (track_finished_tx, _) = watch::channel(());
        let (done_buffering_tx, _) = watch::channel(());

        Ok(Self {
            sink,
            stream_handle,
            sender,
            current_download: Default::default(),
            track_finished_tx,
            done_buffering_tx,
        })
    }

    pub fn track_finished(&self) -> Receiver<()> {
        self.track_finished_tx.subscribe()
    }

    pub fn done_buffering(&self) -> Receiver<()> {
        self.done_buffering_tx.subscribe()
    }

    pub async fn clear(&mut self) -> Result<()> {
        if let Some(handle) = self.current_download.lock()?.take() {
            handle.abort();
        }

        let volume = self.sink.volume();

        let (sender, receiver) = queue(true);
        let sink = rodio::Sink::connect_new(self.stream_handle.mixer());
        sink.append(receiver);
        sink.set_volume(volume);

        self.sink = sink;
        self.sender = sender;

        Ok(())
    }

    pub fn play(&self) {
        self.sink.play();
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn seek(&self, duration: Duration) -> Result<()> {
        self.sink.try_seek(duration)?;

        Ok(())
    }

    pub fn query_track_url(&self, track_url: &str) -> Result<()> {
        if let Some(handle) = self.current_download.lock()?.take() {
            handle.abort();
        }

        let track_url = track_url.to_string();
        let sender = self.sender.clone();
        let track_finished_tx = self.track_finished_tx.clone();
        let done_buffering_tx = self.done_buffering_tx.clone();

        let handle = tokio::spawn(async move {
            let resp = reqwest::get(&track_url).await.unwrap();
            let cursor = Cursor::new(resp.bytes().await.unwrap());

            let source = DecoderBuilder::new()
                .with_data(cursor)
                .with_seekable(true)
                .build()
                .unwrap();

            let signal = sender.append_with_signal(source);

            done_buffering_tx.send(()).unwrap();

            tokio::task::spawn_blocking(move || {
                if signal.recv().is_ok() {
                    track_finished_tx.send(()).unwrap();
                }
            });
        });

        *self.current_download.lock()? = Some(handle);
        Ok(())
    }

    pub fn set_volume(&self, volume: f32) {
        set_volume(&self.sink, volume);
    }
}

fn set_volume(sink: &rodio::Sink, volume: f32) {
    let volume = volume.clamp(0.0, 1.0).powi(3);
    sink.set_volume(volume);
}
