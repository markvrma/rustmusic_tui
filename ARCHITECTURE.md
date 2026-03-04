# Architecture & Code Walkthrough

This document explains the internal structure of `rustmusic_tui`. It is designed for developers who have some basic Rust knowledge and want to understand how a Terminal User Interface (TUI) application works using the `ratatui` library.

## Project Structure

The project is organized into several modules, each with a specific responsibility:

*   **`src/main.rs`**: The entry point. It sets up the terminal, initializes the application, and runs the main event loop.
*   **`src/app.rs`**: Defines the `App` struct. This is the "brain" of the application, holding all state (songs, selection, current view, audio player).
*   **`src/ui.rs`**: Handles rendering. It tells `ratatui` how to draw the `App` state onto the screen.
*   **`src/library.rs`**: Responsible for scanning the filesystem, reading metadata (ID3 tags), and organizing songs into albums.
*   **`src/audio.rs`**: Wraps the `rodio` library to handle audio playback, pausing, and seeking.
*   **`src/config.rs`**: Manages loading and saving user configuration (like the music directory path).
*   **`src/tui.rs`**: A helper module to setup and restore the terminal (entering raw mode, alternate screen).

---

## 1. The Main Loop (`src/main.rs`)

TUI applications typically run in a loop. In `main.rs`, we do three main things:

1.  **Setup**: We switch the terminal to "Raw Mode" (so we catch key presses directly without hitting Enter) and enter the "Alternate Screen" (a separate buffer so we don't mess up your shell history).
2.  **Loop**:
    *   **Draw**: We call `terminal.draw(...)` to render the UI.
    *   **Handle Events**: We wait for a key press (with a timeout). If a key is pressed (e.g., 'j', 'k', 'Enter'), we call a method on `App` to update its state.
    *   **Tick**: We run periodic updates (like updating the progress bar) regardless of input.
3.  **Restore**: When the loop ends (user presses 'q'), we clean up by leaving raw mode and the alternate screen.

**Key Concept**: The loop runs many times a second. Every iteration, we wipe the screen and redraw it based on the current state.

## 2. Application State (`src/app.rs`)

The `App` struct is the central repository of data. It does **not** know how to draw itself; it only holds data and logic.

```rust
pub struct App {
    pub albums: Vec<Album>,          // The loaded music library
    pub current_view: View,          // Are we looking at the Album List or Song List?
    pub album_list_state: ListState, // Which album is selected?
    pub song_list_state: ListState,  // Which song is selected?
    pub audio_player: AudioPlayer,   // The audio backend
    // ...
}
```

*   **`ListState`**: This is a special `ratatui` type used to keep track of which item is selected in a `List` widget.
*   **Navigation Methods**: Methods like `next_album()` or `play_next()` just modify these variables. They don't touch the screen. The screen updates automatically on the next loop iteration because it reads these variables.

## 3. Rendering the UI (`src/ui.rs`)

This is where `ratatui` shines. We define a function `draw(f: &mut Frame, app: &mut App)`.

### Layout
First, we divide the screen into rectangular chunks using `Layout`:

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Min(0),    // Main area (takes remaining space)
        Constraint::Length(3), // Player bar (fixed height of 3 rows)
    ])
    .split(f.area());
```

We can nest layouts to create complex grids (e.g., splitting the Main Area into Left (List) and Right (Art)).

### Widgets
We render "Widgets" into these chunks. Widgets in `ratatui` are **stateless configuration objects**. You create them, render them, and they are consumed immediately.

*   **`Block`**: Draws borders and titles around areas.
*   **`List`**: Displays the list of albums/songs. We pass it the `ListState` from our `App` so it knows which item to highlight.
*   **`Gauge`**: Used for the progress bar.
*   **`Paragraph`**: Used for text.

**Example flow:**
1.  `ui::draw` checks `app.current_view`.
2.  If it's `AlbumList`, it creates a `List` widget with album names.
3.  It calls `f.render_stateful_widget(list, area, &mut app.album_list_state)`.
4.  `ratatui` calculates the characters needed and writes them to the terminal buffer.

### Ratatui-Image
We use the `ratatui-image` crate to render album art. It's smart:
*   It checks what your terminal supports (Sixel, Kitty graphics, or basic Unicode blocks).
*   It renders the image using the best available method into the assigned rectangle.

## 4. Audio Engine (`src/audio.rs`)

We use `rodio` for playback. The `AudioPlayer` struct wraps `rodio::Sink`.
*   **Sink**: Think of it as a queue. We `append` audio sources (files) to it.
*   **Non-blocking**: `rodio` runs audio in a background thread, so our UI doesn't freeze while music plays.
*   **Mutex**: Since the UI thread reads status (is playing? time?) and the audio thread updates it, we use `Arc<Mutex<...>>` to share data safely.

## 5. Library & Metadata (`src/library.rs`)

This module uses:
*   **`walkdir`**: To recursively find all files in your music directory.
*   **`lofty`**: To open those files and read ID3 tags (Artist, Album, Title) and embedded pictures.
*   It groups songs into `Album` structs so we can display them hierarchically.

---

## Summary for Learners

1.  **State vs. UI**: Keep them separate. `App` holds data; `UI` reads data and draws.
2.  **Immediate Mode**: Don't try to "update" a specific button. Just change the state (e.g., `button_color = Red`), and let the next draw cycle re-render the whole UI with the new color.
3.  **Layouts**: TUI design is mostly about slicing rectangles (constraints) and putting widgets inside them.
