use anyhow::Result;
use image::imageops::FilterType;
use ratatui::widgets::ListState;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use std::time::Duration;

use crate::audio::AudioPlayer;
use crate::config::Config;
use crate::library::{load_library, Album};

#[derive(Clone, Copy, PartialEq)]
pub enum View {
    AlbumList,
    SongList,
}

pub struct App {
    pub albums: Vec<Album>,
    pub audio_player: AudioPlayer,
    pub current_view: View,

    pub album_list_state: ListState,
    pub song_list_state: ListState,

    pub picker: Picker,
    pub current_cover_protocol: Option<StatefulProtocol>,

    pub should_quit: bool,
    pub current_song: Option<(usize, usize)>, // (album_idx, song_idx) playing

    // For progress bar
    pub playback_progress: f64, // 0.0 to 1.0
    pub playback_time: String,  // "mm:ss / mm:ss"
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        let albums = load_library(&config.music_directory)?;
        let audio_player = AudioPlayer::new()?;

        let picker = Picker::from_query_stdio()?;

        // Initialize state
        let mut album_list_state = ListState::default();
        if !albums.is_empty() {
            album_list_state.select(Some(0));
        }

        let mut app = App {
            albums,
            audio_player,
            current_view: View::AlbumList,
            album_list_state,
            song_list_state: ListState::default(),
            picker,
            current_cover_protocol: None,
            should_quit: false,
            current_song: None,
            playback_progress: 0.0,
            playback_time: String::new(),
        };

        app.update_cover();

        Ok(app)
    }

    pub fn update_cover(&mut self) {
        if let Some(idx) = self.album_list_state.selected() {
            if let Some(album) = self.albums.get(idx) {
                if let Some(img) = &album.cover {
                    // Resize to fill a square area (600x600) with high quality filter
                    // This handles cropping to square aspect ratio and anti-aliasing
                    let resized = img.resize_to_fill(600, 600, FilterType::Lanczos3);

                    // Create protocol
                    let protocol = self.picker.new_resize_protocol(resized);
                    self.current_cover_protocol = Some(protocol);
                } else {
                    self.current_cover_protocol = None;
                }
            }
        }
    }

    pub fn on_tick(&mut self) {
        // Update playback progress
        if self.audio_player.is_playing() {
            let progress = self.audio_player.get_progress();
            let total = if let Some((alb_idx, song_idx)) = self.current_song {
                if let Some(alb) = self.albums.get(alb_idx) {
                    if let Some(song) = alb.songs.get(song_idx) {
                        Duration::from_secs(song.duration)
                    } else {
                        Duration::from_secs(0)
                    }
                } else {
                    Duration::from_secs(0)
                }
            } else {
                Duration::from_secs(0)
            };

            if total.as_secs() > 0 {
                self.playback_progress = progress.as_secs_f64() / total.as_secs_f64();
                self.playback_progress = self.playback_progress.clamp(0.0, 1.0);
            } else {
                self.playback_progress = 0.0;
            }

            let p_min = progress.as_secs() / 60;
            let p_sec = progress.as_secs() % 60;
            let t_min = total.as_secs() / 60;
            let t_sec = total.as_secs() % 60;

            self.playback_time = format!("{:02}:{:02} / {:02}:{:02}", p_min, p_sec, t_min, t_sec);
        } else if self.audio_player.is_finished() {
            // Autoplay next song
            self.play_next();
        }
    }

    pub fn play_next(&mut self) {
        if let Some((alb_idx, song_idx)) = self.current_song {
            if let Some(album) = self.albums.get(alb_idx) {
                if song_idx < album.songs.len() - 1 {
                    // Play next song in same album
                    let next_song_idx = song_idx + 1;
                    if let Some(song) = album.songs.get(next_song_idx) {
                        if self.audio_player.play_file(&song.path).is_ok() {
                            self.current_song = Some((alb_idx, next_song_idx));
                            // Update UI selection if we are viewing this album
                            if self.current_view == View::SongList {
                                if let Some(selected_alb_idx) = self.album_list_state.selected() {
                                    if selected_alb_idx == alb_idx {
                                        self.song_list_state.select(Some(next_song_idx));
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Album finished. Loop to first song? Or Stop?
                    // Let's stop for now, or we could go to next album.
                    // Let's implement Next Album logic
                    if alb_idx < self.albums.len() - 1 {
                        let next_alb_idx = alb_idx + 1;
                        if let Some(next_album) = self.albums.get(next_alb_idx) {
                            if let Some(song) = next_album.songs.first() {
                                if self.audio_player.play_file(&song.path).is_ok() {
                                    self.current_song = Some((next_alb_idx, 0));
                                    if self.current_view == View::SongList {
                                        // If we are looking at the old album, maybe switch to new one?
                                        // User experience: if I'm browsing, don't jerk me around.
                                        // But if I'm listening, I want to see what's playing.
                                        // Let's only update selection if we are viewing the *previous* album
                                        if let Some(selected_alb_idx) =
                                            self.album_list_state.selected()
                                        {
                                            if selected_alb_idx == alb_idx {
                                                self.album_list_state.select(Some(next_alb_idx));
                                                self.song_list_state.select(Some(0));
                                                self.update_cover();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Navigation methods...
    pub fn next_album(&mut self) {
        if self.albums.is_empty() {
            return;
        }
        let i = match self.album_list_state.selected() {
            Some(i) => {
                if i >= self.albums.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.album_list_state.select(Some(i));
        self.update_cover();
        // If in song list, reset song selection?
        if self.current_view == View::SongList {
            self.song_list_state.select(Some(0));
        }
    }

    pub fn prev_album(&mut self) {
        if self.albums.is_empty() {
            return;
        }
        let i = match self.album_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.albums.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.album_list_state.select(Some(i));
        self.update_cover();
        if self.current_view == View::SongList {
            self.song_list_state.select(Some(0));
        }
    }

    pub fn next_song(&mut self) {
        if let Some(alb_idx) = self.album_list_state.selected() {
            if let Some(album) = self.albums.get(alb_idx) {
                if album.songs.is_empty() {
                    return;
                }
                let i = match self.song_list_state.selected() {
                    Some(i) => {
                        if i >= album.songs.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.song_list_state.select(Some(i));
            }
        }
    }

    pub fn prev_song(&mut self) {
        if let Some(alb_idx) = self.album_list_state.selected() {
            if let Some(album) = self.albums.get(alb_idx) {
                if album.songs.is_empty() {
                    return;
                }
                let i = match self.song_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            album.songs.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.song_list_state.select(Some(i));
            }
        }
    }

    pub fn enter_action(&mut self) {
        match self.current_view {
            View::AlbumList => {
                if !self.albums.is_empty() {
                    self.current_view = View::SongList;
                    self.song_list_state.select(Some(0));
                }
            }
            View::SongList => {
                // Play song
                if let Some(alb_idx) = self.album_list_state.selected() {
                    if let Some(song_idx) = self.song_list_state.selected() {
                        if let Some(album) = self.albums.get(alb_idx) {
                            if let Some(song) = album.songs.get(song_idx) {
                                if let Err(e) = self.audio_player.play_file(&song.path) {
                                    // Handle error? Just print to stderr for now
                                    eprintln!("Error playing file: {}", e);
                                } else {
                                    self.current_song = Some((alb_idx, song_idx));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn back_action(&mut self) {
        if self.current_view == View::SongList {
            self.current_view = View::AlbumList;
        }
    }

    pub fn play_pause(&mut self) {
        self.audio_player.toggle_playback();
    }

    pub fn seek_forward(&mut self) {
        self.audio_player.seek(5);
    }

    pub fn seek_backward(&mut self) {
        self.audio_player.seek(-5);
    }
}
