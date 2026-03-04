use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

mod app;
mod audio;
mod config;
mod library;
mod tui;
mod ui;

use app::{App, View};
use config::Config;

fn main() -> Result<()> {
    // Load config first (might prompt user)
    let config = Config::load()?;

    // Setup terminal
    let mut terminal = tui::init()?;

    // Create app
    let app_result = App::new(config);

    if let Err(e) = app_result {
        tui::restore()?;
        eprintln!("Error initializing application: {}", e);
        return Ok(());
    }
    let mut app = app_result.unwrap();

    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    tui::restore()?;

    if let Err(e) = res {
        eprintln!("Application error: {}", e);
    }

    Ok(())
}

fn run_app(terminal: &mut tui::Tui, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if crossterm::event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('j') => match app.current_view {
                            View::AlbumList => app.next_album(),
                            View::SongList => app.next_song(),
                        },
                        KeyCode::Char('k') => match app.current_view {
                            View::AlbumList => app.prev_album(),
                            View::SongList => app.prev_song(),
                        },
                        KeyCode::Char('J') => {
                            if app.current_view == View::SongList {
                                app.back_action();
                            }
                        }
                        KeyCode::Char('H') => {
                            if app.current_view == View::SongList {
                                app.prev_album();
                            }
                        }
                        KeyCode::Char('L') => {
                            if app.current_view == View::SongList {
                                app.next_album();
                            }
                        }
                        KeyCode::Char('h') => {
                            app.seek_backward();
                        }
                        KeyCode::Char('l') => {
                            app.seek_forward();
                        }
                        KeyCode::Enter => {
                            app.enter_action();
                        }
                        KeyCode::Char(' ') => {
                            app.play_pause();
                        }
                        _ => {}
                    }
                }
            }
        }

        app.on_tick();

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
