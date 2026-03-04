# RustMusic TUI

A terminal-based music player inspired by `cmus`, built with Rust and `ratatui`.

## Features
*   **Vim-like Navigation:** Navigate your library using `h`, `j`, `k`, `l` and other familiar bindings.
*   **Album Art:** Displays pixelated album art in the terminal using block characters.
*   **Library Management:** recursively scans your music directory for MP3, FLAC, OGG, M4A, and WAV files.
*   **Metadata Support:** Reads ID3 tags and embedded cover art.

## Prerequisites
*   **Rust & Cargo:** Ensure you have the latest stable Rust installed.
*   **System Dependencies:** You may need ALSA development headers on Linux.
    *   Ubuntu/Debian: `sudo apt install libasound2-dev`
    *   Fedora: `sudo dnf install alsa-lib-devel`

## Installation & Running

1.  Clone the repository:
    ```bash
    git clone <repository_url>
    cd rustmusic_tui
    ```

2.  Build and Run:
    ```bash
    cargo run --release
    ```

## Configuration
On the first run, the application will ask for the absolute path to your music directory (e.g., `/home/user/Music`). This path is saved to `~/.config/rustmusic_tui/config.toml`.

To change the library path later, you can edit this file directly.

## Usage / Keybindings

| Key | Action | Context |
| :--- | :--- | :--- |
| `j` / `k` | Navigate Down / Up | Album List & Song List |
| `Enter` | Open Album / Play Song | Album List / Song List |
| `Shift+J` (`J`) | Go Back to Album List | Song List |
| `Shift+H` (`H`) | Jump to Previous Album | Song List |
| `Shift+L` (`L`) | Jump to Next Album | Song List |
| `Space` | Play / Pause | Global |
| `h` / `l` | Seek Backward / Forward | Global (Currently Disabled) |
| `q` / `Esc` | Quit | Global |

## Troubleshooting
*   **Audio Issues:** Ensure no other application has exclusive lock on the audio device if using ALSA directly.
*   **Album Art:** If images look distorted, ensure your terminal font supports block characters properly. The player looks for embedded art first, then `cover.jpg`, `folder.jpg`, etc., in the album directory.
