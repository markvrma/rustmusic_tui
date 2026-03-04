use anyhow::Result;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlayerState {
    Playing,
    Paused,
    Stopped,
}

struct AudioStatus {
    state: PlayerState,
    start_time: Option<Instant>,
    elapsed_before_pause: Duration,
}

pub struct AudioPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink,
    status: Arc<Mutex<AudioStatus>>,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        Ok(AudioPlayer {
            _stream,
            stream_handle,
            sink,
            status: Arc::new(Mutex::new(AudioStatus {
                state: PlayerState::Stopped,
                start_time: None,
                elapsed_before_pause: Duration::ZERO,
            })),
        })
    }

    pub fn play_file(&mut self, path: &Path) -> Result<()> {
        // Create a new sink to ensure clean state
        if !self.sink.empty() {
            self.sink.stop();
        }
        self.sink = Sink::try_new(&self.stream_handle)?;

        let file = File::open(path)?;
        let source = Decoder::new(BufReader::new(file))?;

        self.sink.append(source);
        // self.sink.play(); // sink plays by default when appended if not paused? verify.
        // rodio 0.17+: append adds to queue. play() resumes if paused.
        // Newly created sink is not paused.

        let mut status = self.status.lock().unwrap();
        status.state = PlayerState::Playing;
        status.start_time = Some(Instant::now());
        status.elapsed_before_pause = Duration::ZERO;

        Ok(())
    }

    pub fn pause(&mut self) {
        if !self.sink.is_paused() {
            self.sink.pause();
            let mut status = self.status.lock().unwrap();
            status.state = PlayerState::Paused;
            if let Some(start) = status.start_time {
                status.elapsed_before_pause += start.elapsed();
            }
            status.start_time = None;
        }
    }

    pub fn resume(&mut self) {
        if self.sink.is_paused() {
            self.sink.play();
            let mut status = self.status.lock().unwrap();
            status.state = PlayerState::Playing;
            status.start_time = Some(Instant::now());
        }
    }

    pub fn toggle_playback(&mut self) {
        if self.sink.empty() {
            return;
        }
        if self.sink.is_paused() {
            self.resume();
        } else {
            self.pause();
        }
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        let mut status = self.status.lock().unwrap();
        status.state = PlayerState::Stopped;
        status.start_time = None;
        status.elapsed_before_pause = Duration::ZERO;
    }

    pub fn seek(&mut self, seconds: i64) {
        let current = self.get_progress();
        let new_pos = if seconds >= 0 {
            current + Duration::from_secs(seconds as u64)
        } else {
            let sub = (-seconds) as u64;
            if current.as_secs() < sub {
                Duration::ZERO
            } else {
                current - Duration::from_secs(sub)
            }
        };

        if self.sink.try_seek(new_pos).is_ok() {
            let mut status = self.status.lock().unwrap();
            if status.state == PlayerState::Playing {
                status.start_time = Some(Instant::now());
            }
            status.elapsed_before_pause = new_pos;
        }
    }

    pub fn get_progress(&self) -> Duration {
        if self.sink.empty() {
            return Duration::ZERO;
        }

        let status = self.status.lock().unwrap();
        match status.state {
            PlayerState::Playing => {
                if let Some(start) = status.start_time {
                    status.elapsed_before_pause + start.elapsed()
                } else {
                    status.elapsed_before_pause
                }
            }
            _ => status.elapsed_before_pause,
        }
    }

    pub fn is_playing(&self) -> bool {
        let status = self.status.lock().unwrap();
        !self.sink.empty() && status.state == PlayerState::Playing
    }
}
