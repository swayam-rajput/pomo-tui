use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::{App, Mode, Screen};

 // ─── RENDER ENTRY POINT ──────────────────────────────────────────────────────
// ratatui calls this every ~50ms with a fresh Frame to draw into.
// We decide which screen to show based on `app.screen`.
pub fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Timer => render_timer(frame, app),
        Screen::Settings => render_settings(frame, app),
    }
}

// ─── TIMER SCREEN ────────────────────────────────────────────────────────────
fn render_timer(frame: &mut Frame, app: &App) {
    // Split the terminal into 5 rows.
    // Think of this like CSS Flexbox — each row gets a fixed or flexible size.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // "we are working 🎯"
            Constraint::Length(1), // blank spacer
            Constraint::Length(1), // "24:07  (sessions: 3)"
            Constraint::Length(1), // blank spacer
            Constraint::Length(3), // [progress bar]
            Constraint::Min(0),    // fills remaining space
            Constraint::Length(1), // help bar
        ])
        .split(frame.area());

    let color = mode_color(&app.mode);

    // ── Row 0: Mode label ────────────────────────────────────────────────
    frame.render_widget(
        Paragraph::new(app.mode.label())
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
        rows[0],
    );

    // ── Row 2: Time remaining + session count ────────────────────────────
    // `Line::from(vec![...])` builds a single line from multiple styled spans.
    // This lets different parts of the same line have different colors.
    let time_line = Line::from(vec![
        Span::styled(
            app.time_str(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            format!("sessions done: {}", app.sessions_done),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(time_line), rows[2]);

    // ── Row 4: Progress bar ──────────────────────────────────────────────
    // `app.progress()` gives us a fresh f64 (0.0–1.0) computed from the
    // real wall clock — that's what makes the bar move smoothly between ticks.
    let percent = (app.progress() * 100.0) as u16;

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(
            Style::default()
                .fg(color)
                .bg(Color::Rgb(35, 35, 50)),
        )
        .ratio(app.progress())
        .label(format!("{}%", percent));

    frame.render_widget(gauge, rows[4]);

    // ── Row 6: Help bar ──────────────────────────────────────────────────
    let status = if app.is_running {
        Span::styled("▶ running", Style::default().fg(Color::Green))
    } else {
        Span::styled("⏸ paused ", Style::default().fg(Color::Yellow))
    };

    let help = Line::from(vec![
        status,
        Span::raw("  [space] play/pause  [r] reset  [n] skip  [t] settings  [q] quit"),
    ]);
    frame.render_widget(Paragraph::new(help), rows[6]);
}

// ─── SETTINGS SCREEN ─────────────────────────────────────────────────────────
fn render_settings(frame: &mut Frame, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(1), // spacer
            Constraint::Length(1), // Work duration row
            Constraint::Length(1), // Short break row
            Constraint::Length(1), // Long break row
            Constraint::Min(0),    // spacer
            Constraint::Length(1), // Help bar
        ])
        .split(frame.area());

    // ── Title ────────────────────────────────────────────────────────────
    frame.render_widget(
        Paragraph::new("⚙  Settings")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        rows[0],
    );

    // ── The three setting rows ────────────────────────────────────────────
    // We zip the index, label, and value together and render each row.
    let items = [
        ("Work         ", app.work_secs / 60),
        ("Short Break  ", app.short_break_secs / 60),
        ("Long Break   ", app.long_break_secs / 60),
    ];

    for (i, (label, minutes)) in items.iter().enumerate() {
        // Is this the currently selected row? Show it highlighted.
        let is_selected = i == app.settings_idx;

        let prefix = if is_selected { "▶ " } else { "  " };

        let line = Line::from(vec![
            Span::styled(
                format!("{}{}", prefix, label),
                if is_selected {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                },
            ),
            Span::styled(
                format!("{:2} min", minutes),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            // Show hint only on the selected row
            if is_selected {
                Span::styled(
                    "  ← → to adjust",
                    Style::default().fg(Color::DarkGray),
                )
            } else {
                Span::raw("")
            },
        ]);

        // rows[2] is item 0, rows[3] is item 1, rows[4] is item 2
        frame.render_widget(Paragraph::new(line), rows[2 + i]);
    }

    // ── Help bar ─────────────────────────────────────────────────────────
    frame.render_widget(
        Paragraph::new("  [↑↓] select  [←→] adjust  [t / enter] back to timer"),
        rows[6],
    );
}

// ─── HELPER ──────────────────────────────────────────────────────────────────
fn mode_color(mode: &Mode) -> Color {
    match mode {
        Mode::Work => Color::Rgb(160, 100, 255),       // Purple
        Mode::ShortBreak => Color::Rgb(80, 210, 140),  // Mint
        Mode::LongBreak => Color::Rgb(80, 170, 255),   // Sky blue
    }
}
