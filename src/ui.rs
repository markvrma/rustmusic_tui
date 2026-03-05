use crate::app::{App, View};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem},
    Frame,
};
use ratatui_image::StatefulImage;

pub fn draw(f: &mut Frame, app: &mut App) {
    // Render background
    let bg_block = Block::default().style(Style::default().bg(app.bg_color));
    f.render_widget(bg_block, f.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let main_area = chunks[0];
    let bottom_area = chunks[1];

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_area);

    let left_area = main_chunks[0];
    let right_area = main_chunks[1];

    draw_list(f, app, left_area);
    draw_art(f, app, right_area);
    draw_player(f, app, bottom_area);
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let title = match app.current_view {
        View::AlbumList => " Albums ".to_string(),
        View::SongList => {
            if let Some(idx) = app.album_list_state.selected() {
                if let Some(album) = app.albums.get(idx) {
                    format!(" {} ", album.title)
                } else {
                    " Songs ".to_string()
                }
            } else {
                " Songs ".to_string()
            }
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme_color))
        .title(Line::from(title).style(Style::default().fg(app.theme_color)));

    match app.current_view {
        View::AlbumList => {
            let items: Vec<ListItem> = app
                .albums
                .iter()
                .map(|album| ListItem::new(format!("{} - {}", album.artist, album.title)))
                .collect();

            let list = List::new(items)
                .block(block)
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::REVERSED)
                        .fg(app.theme_color),
                )
                .highlight_symbol("> ");

            f.render_stateful_widget(list, area, &mut app.album_list_state);
        }
        View::SongList => {
            if let Some(idx) = app.album_list_state.selected() {
                if let Some(album) = app.albums.get(idx) {
                    let items: Vec<ListItem> = album
                        .songs
                        .iter()
                        .enumerate()
                        .map(|(i, song)| {
                            let duration =
                                format!("{}:{:02}", song.duration / 60, song.duration % 60);
                            let title = format!("{}. {} ({})", i + 1, song.title, duration);
                            ListItem::new(title)
                        })
                        .collect();

                    let list = List::new(items)
                        .block(block)
                        .highlight_style(
                            Style::default()
                                .add_modifier(Modifier::REVERSED)
                                .fg(app.theme_color),
                        )
                        .highlight_symbol("> ");

                    f.render_stateful_widget(list, area, &mut app.song_list_state);
                } else {
                    f.render_widget(block, area);
                }
            } else {
                f.render_widget(block, area);
            }
        }
    }
}

fn draw_art(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme_color))
        .title(Line::from(" Album Art ").style(Style::default().fg(app.theme_color)));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if let Some(protocol) = &mut app.current_cover_protocol {
        let image = StatefulImage::default();
        f.render_stateful_widget(image, inner_area, protocol);
    }
}

fn draw_player(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme_color))
        .title(Line::from(" Player ").style(Style::default().fg(app.theme_color)));

    let label = if let Some((alb_idx, song_idx)) = app.current_song {
        if let Some(alb) = app.albums.get(alb_idx) {
            if let Some(song) = alb.songs.get(song_idx) {
                format!("{} - {}  [{}]", song.artist, song.title, app.playback_time)
            } else {
                app.playback_time.clone()
            }
        } else {
            app.playback_time.clone()
        }
    } else {
        "Stopped".to_string()
    };

    let gauge = Gauge::default()
        .block(block)
        .gauge_style(Style::default().fg(app.theme_color))
        .ratio(app.playback_progress)
        .label(Span::styled(label, Style::default().fg(Color::White)));

    f.render_widget(gauge, area);
}
