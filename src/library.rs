use anyhow::{Context, Result};
use image::{DynamicImage, ImageReader};
use lofty::{
    file::{AudioFile, TaggedFileExt},
    probe::Probe,
    tag::{Accessor, Tag},
};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct Song {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: u64, // duration in seconds
    pub track_number: Option<u32>,
}

#[derive(Clone)]
pub struct Album {
    pub title: String,
    pub artist: String,
    pub year: Option<u32>,
    pub songs: Vec<Song>,
    pub cover: Option<DynamicImage>,
}

pub fn load_library(root: &Path) -> Result<Vec<Album>> {
    let mut albums: HashMap<(String, String), Album> = HashMap::new();

    for entry in WalkDir::new(root).follow_links(true) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext.to_lowercase().as_str() {
                "mp3" | "flac" | "ogg" | "m4a" | "wav" => {
                    // We use a match to safely handle the Result from Probe
                    if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) {
                        let tag = tagged_file
                            .primary_tag()
                            .or_else(|| tagged_file.first_tag());

                        let title = tag
                            .and_then(|t| t.title().map(|s| s.to_string()))
                            .unwrap_or_else(|| {
                                path.file_stem().unwrap().to_string_lossy().to_string()
                            });

                        let artist = tag
                            .and_then(|t| t.artist().map(|s| s.to_string()))
                            .unwrap_or_else(|| "Unknown Artist".to_string());

                        let album_title = tag
                            .and_then(|t| t.album().map(|s| s.to_string()))
                            .unwrap_or_else(|| "Unknown Album".to_string());

                        let year = tag.and_then(|t| t.year());
                        let track_number = tag.and_then(|t| t.track());

                        let duration = tagged_file.properties().duration().as_secs();

                        let song = Song {
                            path: path.to_path_buf(),
                            title,
                            artist: artist.clone(),
                            album: album_title.clone(),
                            duration,
                            track_number,
                        };

                        let key = (album_title.clone(), artist.clone());

                        if let Some(album) = albums.get_mut(&key) {
                            album.songs.push(song);
                            if album.cover.is_none() {
                                album.cover = extract_cover(path, tag);
                            }
                        } else {
                            let cover = extract_cover(path, tag);
                            let album = Album {
                                title: album_title,
                                artist,
                                year,
                                songs: vec![song],
                                cover,
                            };
                            albums.insert(key, album);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Sort songs in albums and convert to Vec
    let mut album_list: Vec<Album> = albums.into_values().collect();
    for album in &mut album_list {
        album.songs.sort_by_key(|s| s.track_number.unwrap_or(0));
    }

    // Sort albums by artist then title
    album_list.sort_by(|a, b| a.artist.cmp(&b.artist).then(a.title.cmp(&b.title)));

    Ok(album_list)
}

fn extract_cover(path: &Path, tag: Option<&impl Accessor>) -> Option<DynamicImage> {
    // 1. Try embedded art
    if let Some(tag) = tag {
        if let Some(picture) = tag.pictures().first() {
            if let Ok(img) = ImageReader::new(Cursor::new(picture.data()))
                .with_guessed_format()
                .ok()?
                .decode()
            {
                return Some(img);
            }
        }
    }

    // 2. Try cover.jpg / folder.jpg in the same directory
    if let Some(parent) = path.parent() {
        let candidates = [
            "cover.jpg",
            "folder.jpg",
            "cover.png",
            "folder.png",
            "front.jpg",
        ];
        for candidate in candidates {
            let p = parent.join(candidate);
            if p.exists() {
                if let Ok(img) = ImageReader::open(p).ok()?.decode() {
                    return Some(img);
                }
            }
        }
    }

    None
}
