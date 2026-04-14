// ratatui gives us everything we need to build the TUI layout and widgets.
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

// We import `App` from our sibling module `app.rs`.
// `crate::` means "start from the root of this project".
use crate::app::{App, Mode};

// ─── RENDER ──────────────────────────────────────────────────────────────────
// This is the only public function in ui.rs.
// `frame` is a mutable reference to the drawing surface.
// `app` is a shared (read-only) reference to our state.
// We intentionally only READ from `app` here — the UI should never change state.
pub fn render(frame: &mut Frame, app: &App) {
    // ── 1. SPLIT THE SCREEN INTO ROWS ────────────────────────────────────
    // `Layout` divides the terminal area into chunks, like CSS Flexbox.
    // Direction::Vertical means we're stacking rows top-to-bottom.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Mode label  ("we are working 🎯")
            Constraint::Length(1), // Time string  ("24:07")
            Constraint::Length(2), // Progress bar
            Constraint::Min(0),    // Empty space (fills whatever is left)
            Constraint::Length(1), // Help bar at the bottom
        ])
        .split(frame.area());

    // ── 2. MODE LABEL ────────────────────────────────────────────────────
    // `Paragraph` is a text widget. We style it with the mode's color.
    let label_color = mode_color(&app.mode);

    let mode_label = Paragraph::new(app.mode.label())
        .style(
            Style::default()
                .fg(label_color)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Left);

    // `render_widget` draws the widget into a specific chunk of the screen.
    frame.render_widget(mode_label, rows[0]);

    // ── 3. TIME REMAINING ────────────────────────────────────────────────
    // A simple one-line text showing "24:07".
    let time_widget = Paragraph::new(Span::styled(
        app.time_str(),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Left);

    frame.render_widget(time_widget, rows[1]);

    // ── 4. PROGRESS BAR ──────────────────────────────────────────────────
    // `Gauge` is ratatui's built-in progress bar widget.
    // `.ratio()` takes a f64 between 0.0 and 1.0.
    let percent = (app.progress() * 100.0) as u16;

    let gauge = Gauge::default()
        // No border — just the bare bar, like in the screenshot.
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(
            Style::default()
                .fg(mode_color(&app.mode))
                .bg(Color::Rgb(40, 40, 55)), // Dark unfilled background
        )
        .ratio(app.progress())
        // The label shown in the center of the bar.
        .label(format!("{}%", percent));

    frame.render_widget(gauge, rows[2]);

    // ── 5. HELP BAR ──────────────────────────────────────────────────────
    // Shows the available keybinds at the bottom.
    let status = if app.is_running { "▶ running" } else { "⏸ paused" };

    // `Line::from` builds a line from multiple `Span`s (each can have its own style).
    let help = Line::from(vec![
        Span::styled(status, Style::default().fg(Color::Yellow)),
        Span::raw("  [space] play/pause  [r] reset  [s] skip  [q] quit"),
    ]);

    frame.render_widget(
        Paragraph::new(help).alignment(Alignment::Left),
        rows[4],
    );
}

// ─── HELPER ──────────────────────────────────────────────────────────────────
// A small private helper that picks a color for the current mode.
// It's private (no `pub`) because only this file needs it.
fn mode_color(mode: &Mode) -> Color {
    match mode {
        Mode::Work => Color::Rgb(160, 100, 255), // Purple
        Mode::ShortBreak => Color::Rgb(100, 220, 160), // Mint green
        Mode::LongBreak => Color::Rgb(100, 180, 255),  // Sky blue
    }
}
