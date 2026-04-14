// ui.rs
//
// This module owns all rendering. It receives a read-only reference to App
// and draws whatever the current state calls for. It never mutates anything.
//
// The central concept in ratatui: you describe the UI from scratch every frame.
// There is no diffing, no retained state. Every tick you call render() and the
// library figures out what changed. This is called "immediate mode" rendering.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
};

use crate::app::{App, Phase, TimerState};

// The ludicrous bar uses these Unicode block characters to fill itself.
// Each tick we pick from different positions to make it look alive.
const FILL_CHARS: &[char] = &[
    '\u{2588}', // FULL BLOCK        ████
    '\u{2593}', // DARK SHADE        ▓▓▓▓
    '\u{2592}', // MEDIUM SHADE      ▒▒▒▒
    '\u{2591}', // LIGHT SHADE       ░░░░
    // '\u{25A0}', // BLACK SQUARE      ■■■■
    // '\u{25CF}', // BLACK CIRCLE      ●●●●
    // '\u{2665}', // HEART             ♥♥♥♥  (for break phases)
    // '\u{2605}', // STAR              ★★★★
    // '\u{26A1}', // LIGHTNING         ⚡⚡⚡  (for final stretch)
];

// Braille density characters, low to high density.
// These are used in the "shimmer" zone just ahead of the fill line.
const BRAILLE: &[char] = &[
    '\u{2801}', '\u{2803}', '\u{2807}', '\u{280F}',
    '\u{281F}', '\u{283F}', '\u{287F}', '\u{28FF}',
];

// Color palette for the progress bar gradient.
// The bar shifts through these colors as progress increases.
// We index into this based on progress percentage.
fn progress_color(progress: f64, tick: u64) -> Color {
    let urgency = progress > 0.85;
    let nearly_done = progress > 0.95;

    if nearly_done {
        // Flicker between red and bright yellow in the final stretch
        if (tick / 3) % 2 == 0 {
            Color::Rgb(255, 50, 50)
        } else {
            Color::Rgb(255, 220, 0)
        }
    } else if urgency {
        Color::Rgb(255, 120, 0) // orange warning
    } else {
        // Smooth gradient: green -> cyan -> blue as progress goes 0->1
        let t = progress as f32;
        let r = (0.0 * (1.0 - t) + 30.0 * t) as u8;
        let g = (220.0 * (1.0 - t) + 180.0 * t) as u8;
        let b = (100.0 * (1.0 - t) + 255.0 * t) as u8;
        Color::Rgb(r, g, b)
    }
}

fn bg_color(phase: &Phase) -> Color {
    match phase {
        Phase::Work => Color::Rgb(20, 20, 35),
        Phase::ShortBreak => Color::Rgb(15, 30, 20),
        Phase::LongBreak => Color::Rgb(25, 15, 35),
    }
}

// The actual chaos bar renderer.
// We build a Vec<Span> (styled text segments) that together fill the bar area.
//
// Layout of the bar for a given progress p (bar width = W):
//
//   [  filled zone  |shimmer| empty zone  ]
//    0 ............. p*W ..... W
//
// The shimmer zone is ~3 chars wide and lives right at the fill boundary.
// The filled zone uses animated characters that cycle per-tick.
// The empty zone uses a faint░░░ shade to show where the bar will go.
pub fn chaos_bar_spans(progress: f64, tick: u64, width: u16, phase: &Phase) -> Vec<Span<'static>> {
    let w = width as usize;
    if w == 0 {
        return vec![];
    }

    let filled = ((progress * w as f64).floor() as usize).min(w);
    let shimmer_start = filled.saturating_sub(0); // shimmer IS the leading edge
    let shimmer_len = if filled < w { 3_usize.min(w - filled) } else { 0 };

    let fg = progress_color(progress, tick);
    let fill_style = Style::default().fg(fg).add_modifier(Modifier::BOLD);
    let shimmer_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let empty_style = Style::default().fg(Color::Rgb(50, 50, 70));

    let mut spans: Vec<Span<'static>> = Vec::new();

    // Filled zone: characters cycle through FILL_CHARS per tick.
    // Using a different cycle speed per character position creates a
    // "flowing lava" effect -- each column animates independently.
    if filled > 0 {
        let fill_chars: String = (0..filled)
            .map(|i| {
                // Each column has a phase offset based on its position.
                // Dividing tick by different amounts gives different speeds.
                let char_index = ((tick / 2) + i as u64) as usize;

                // In the final 5% show only lightning bolts
                let near_end = (i as f64 / w as f64) > 0.92 && progress > 0.92;

                // During break phases bias toward hearts/stars
                let chars = if near_end {
                    &FILL_CHARS[0..1] // lightning only
                } else {
                    match phase {
                        Phase::Work => &FILL_CHARS[..],
                        Phase::ShortBreak | Phase::LongBreak => &FILL_CHARS[1..],
                    }
                };

                chars[char_index % chars.len()]
            })
            .collect();

        spans.push(Span::styled(fill_chars, fill_style));
    }

    // Shimmer zone: braille characters right at the leading edge.
    // These are brighter than the fill and flicker faster.
    // if shimmer_len > 0 {
    //     let shimmer_chars: String = (0..shimmer_len)
    //         .map(|i| {
    //             let idx = ((tick * 3) + i as u64) as usize % BRAILLE.len();
    //             BRAILLE[idx]
    //         })
    //         .collect();
    //     spans.push(Span::styled(shimmer_chars, shimmer_style));
    // }

    // Empty zone: remainder of the bar, faint block characters
    let empty_len = w.saturating_sub(filled + shimmer_len);
    if empty_len > 0 {
        let empty_chars: String = std::iter::repeat('\u{2591}').take(empty_len).collect();
        spans.push(Span::styled(empty_chars, empty_style));
    }

    spans
}

// The main render function. Called every tick by main.rs.
pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // ---- Layout ----
    // We split the screen vertically into zones.
    // Constraint::Length = fixed number of rows
    // Constraint::Min    = fills remaining space
    // Constraint::Percentage = fraction of space
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Min(1),       // top spacer
            Constraint::Length(3),    // phase label + pomodoro count
            Constraint::Length(5),    // big timer display
            Constraint::Length(5),    // the chaos bar
            Constraint::Length(3),    // status / done message
            Constraint::Length(2),    // keybind hints
            Constraint::Min(1),       // bottom spacer
        ])
        .split(size);

    let bg_col = bg_color(&app.phase);

    // ---- Phase label row ----
    let phase_color = match app.phase {
        Phase::Work => Color::Rgb(100, 200, 255),
        Phase::ShortBreak => Color::Rgb(100, 255, 150),
        Phase::LongBreak => Color::Rgb(200, 150, 255),
    };

    let pomodoro_dots: String = {
        let done = app.pomodoros_done % 3;
        let filled = "\u{25CF}".repeat(done as usize);      // ●
        let empty  = "\u{25CB}".repeat((3 - done) as usize); // ○
        format!("{}{}", filled, empty)
    };

    let phase_line = Line::from(vec![
        Span::styled(
            format!("  {}  ", app.phase.label()),
            Style::default()
                .fg(bg_col)
                .bg(phase_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(pomodoro_dots, Style::default().fg(phase_color)),
    ]);

    f.render_widget(
        Paragraph::new(phase_line).alignment(Alignment::Center),
        outer_chunks[1],
    );

    // ---- Timer display ----
    let (mins, secs) = app.remaining();
    let timer_str = format!("{:02}:{:02}", mins, secs);

    // Pulsing effect: slightly dim the timer when paused
    let timer_style = if app.state == TimerState::Paused {
        Style::default()
            .fg(Color::Rgb(120, 120, 140))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    };

    // Build timer as individual characters with per-character style variation
    // to make it feel "alive" in the last 60 seconds.
    let timer_widget = if app.progress() > 0.9 && app.state == TimerState::Running {
        // In the final 10%, each digit pulses slightly independently
        let chars: Vec<Span> = timer_str
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let pulse = ((app.tick / 4) + i as u64) % 8 == 0;
                let style = if pulse {
                    Style::default()
                        .fg(Color::Rgb(255, 80, 80))
                        .add_modifier(Modifier::BOLD)
                } else {
                    timer_style
                };
                Span::styled(c.to_string(), style)
            })
            .collect();
        Paragraph::new(Line::from(chars))
    } else {
        Paragraph::new(Span::styled(timer_str, timer_style))
    };

    f.render_widget(
        timer_widget
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE)),
        outer_chunks[2],
    );

    // ---- The Chaos Bar ----
    // We render this inside a Block (gives us a border + title for free)
    // then manually place the custom spans inside the inner area.
    let bar_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 80)))
        .title(Span::styled(
            format!(" {:.0}% ", app.progress() * 100.0),
            Style::default().fg(Color::Rgb(160, 160, 180)),
        ))
        .title_alignment(Alignment::Right);

    // Get the inner area of the block (subtracts borders)
    let bar_inner = bar_block.inner(outer_chunks[3]);
    f.render_widget(bar_block, outer_chunks[3]);

    // Now render our custom spans into bar_inner.
    // We center the bar vertically in the inner area.
    let bar_row = Rect {
        x: bar_inner.x,
        y: bar_inner.y + bar_inner.height / 2,
        width: bar_inner.width,
        height: 1,
    };

    let bar_spans = chaos_bar_spans(app.progress(), app.tick, bar_row.width, &app.phase);
    let bar_line = Line::from(bar_spans);
    f.render_widget(Paragraph::new(bar_line), bar_row);

    // ---- Status line ----
    let status_widget = match app.state {
        TimerState::Running => {
            let msg = if app.progress() > 0.9 {
                // Increasingly frantic messages as time runs out
                match ((app.progress() - 0.9) * 100.0) as u32 {
                    0..=3  => "almost there...",
                    4..=6  => "keep going!!",
                    7..=8  => "DO NOT STOP",
                    _      => "FINISH IT",
                }
            } else {
                "stay focused"
            };
            Paragraph::new(Span::styled(
                msg,
                Style::default().fg(Color::Rgb(100, 100, 120)).add_modifier(Modifier::ITALIC),
            ))
            .alignment(Alignment::Center)
        }

        TimerState::Paused => Paragraph::new(Line::from(vec![
            Span::styled(
                "  PAUSED  ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(200, 180, 80))
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Center),

        TimerState::Done => {
            let msg = match app.phase {
                Phase::Work => "session complete. take a breath.",
                Phase::ShortBreak | Phase::LongBreak => "break over. back to it.",
            };
            Paragraph::new(Span::styled(
                msg,
                Style::default()
                    .fg(Color::Rgb(100, 255, 180))
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center)
        }
    };
    f.render_widget(status_widget, outer_chunks[4]);

    // ---- Keybind hints ----
    let hints = match app.state {
        TimerState::Done => "[enter] next phase   [q] quit",
        _ => "[space] pause/resume   [s] skip   [q] quit",
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            hints,
            Style::default().fg(Color::Rgb(60, 60, 80)),
        ))
        .alignment(Alignment::Center),
        outer_chunks[5],
    );
}
