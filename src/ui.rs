
use ratatui::{
    Frame, 
    layout::{ Alignment, Constraint, Direction, Layout, Rect }, 
    style::{Color, Modifier, Style}, 
    text::{Line, Span}, 
    widgets::{Block, Borders, Paragraph}
};

use crate::app::{App, NotificationMode, Phase, Screen, TimerState};

const FILL_CHARS: &[char] = &[
    '\u{2588}', // FULL BLOCK        ████
];
const RED:    Color = Color::Rgb(123, 204, 140);
// const RED:    Color = Color::Rgb(121, 174, 111);
const ORANGE: Color = Color::Rgb(242, 153,  74);
const GREEN:  Color = Color::Rgb(164, 188, 224);
const GRAY:   Color = Color::Rgb(120, 120, 130);
// const WHITE:  Color = Color::Rgb(230, 230, 240);
// const DIM:    Color = Color::Rgb( 80,  80,  90);
const BG:     Color = Color::Rgb( 18,  18,  24);

fn phase_color(phase: &Phase) -> Color {
    match phase {
        Phase::Work       => RED,
        Phase::ShortBreak => GREEN,
        Phase::LongBreak  => ORANGE,
    }
}


fn timer_fg(state: &TimerState, phase: &Phase) -> Color {
    match state {
        TimerState::Done   => phase_color(phase),
        TimerState::Paused => GRAY,
        TimerState::Running => phase_color(phase),
    }
}


pub fn progress_animation(progress: f64, tick: u64, width: u16, phase: &Phase) -> Vec<Span<'static>>{
    let w = width as usize;
    if w == 0 {
        return vec![];
    }
    
    let filled = ((progress * w as f64).floor() as usize).min(w);
    let fg = phase_color(&phase);
    let shimmer_len = if filled < w { 3_usize.min(w - filled) } else { 0 };
    let fill_style = Style::default().fg(fg).add_modifier(Modifier::BOLD);
    let empty_style = Style::default().fg(Color::Rgb(50, 50, 70));

    let mut spans: Vec<Span<'static>> = Vec::new();
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
                    &FILL_CHARS[0..] // lightning only
                } else {
                    match phase {
                        Phase::Work => &FILL_CHARS[..],
                        Phase::ShortBreak | Phase::LongBreak => &FILL_CHARS[0..],
                    }
                };

                chars[char_index % chars.len()]
            })
            .collect();

        spans.push(Span::styled(fill_chars, fill_style));
    }
    let empty_len = w.saturating_sub(filled + shimmer_len);
    if empty_len > 0 {
        let empty_chars: String = std::iter::repeat('\u{2591}').take(empty_len).collect();
        spans.push(Span::styled(empty_chars, empty_style));
    }

    
    return spans;




    
}

fn notification_label(mode: NotificationMode) -> &'static str {
    match mode {
        NotificationMode::Off => "Off",
        NotificationMode::WorkOnly => "Work Only",
        NotificationMode::BreakOnly => "Break Only",
        NotificationMode::All => "All",
    }
}

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

    frame.render_widget(
        Paragraph::new("⚙  Settings")
            .style(Style::default().fg(Color::Rgb(111, 207, 151)).add_modifier(Modifier::BOLD)),
        rows[0],
    );

    // ── The three setting rows ────────────────────────────────────────────
    // We zip the index, label, and value together and render each row.
    let items = [
        ("Work         ", format!("{} min", app.work_secs / 60)),
        ("Short Break  ", format!("{} min", app.short_break_secs / 60)),
        ("Long Break   ", format!("{} min", app.long_break_secs / 60)),
        ("Notifications", notification_label(app.notif_mode).to_string()),
    ];

    for (i, (label, value)) in items.iter().enumerate() {
    let is_selected = i == app.settings_idx;
    let prefix = if is_selected { "> " } else { "  " };

    let line = Line::from(vec![
        Span::styled(
            format!("{}{}", prefix, label),
            if is_selected {
                Style::default().fg(Color::Rgb(111, 207, 151)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            },
        ),
        Span::styled(
            format!("   {}", value),
            Style::default()
                .fg(Color::White)
        ),
        if is_selected {
            Span::styled("  ← → to adjust", Style::default().fg(Color::DarkGray))
        } else {
            Span::raw("")
        },
    ]);

    frame.render_widget(Paragraph::new(line), rows[2 + i]);
}
    frame.render_widget(
        Paragraph::new("  [↑↓] select  [←→] adjust  [t / enter] back to timer"),
        rows[6],
    );
}

pub fn render_timer(f: &mut Frame, app: &App){

    
    // ── outer vertical layout ────────────────────────────────────────────────
    //  [0] top padding
    //  [1] phase label          (1 line)
    //  [2] spacer               (1 line)
    //  [3] big timer            (1 line)
    //  [4] spacer               (1 line)
    //  [5] progress gauge       (3 lines)
    //  [6] spacer               (1 line)
    //  [7] status + pomodoros   (1 line)
    //  [8] spacer               (1 line)
    //  [9] keybinds hint        (1 line)
    // [10] bottom padding

    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(3)
    .constraints([
        Constraint::Min(1),       // top spacer
        Constraint::Length(3),    // phase label + pomodoro count
        Constraint::Length(5),    // big timer display
        Constraint::Length(5),    // the chaos bar
        Constraint::Length(3),    // status / done message
        Constraint::Length(2),    // keybind hints
        Constraint::Min(1),   
    ])
    .split(f.area());    


    let bar_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 80)))
        .title(Span::styled(
            format!(" {:.0}% ", app.progress() * 100.0),
            Style::default().fg(Color::Rgb(160, 160, 180)),
        ))
        .title_alignment(Alignment::Right);
    let bar_inner = bar_block.inner(chunks[3]);
    f.render_widget(bar_block, chunks[3]);

    let bar_row = Rect {
        x: bar_inner.x,
        y: bar_inner.y + bar_inner.height / 2,
        width: bar_inner.width,
        height: 1,
    };

    let bar_spans = progress_animation(app.progress(), app.tick, bar_row.width, &app.phase);
    let bar_line = Line::from(bar_spans);
    f.render_widget(Paragraph::new(bar_line), bar_row);

    let (m,s) = app.remaining();
    let time_str = format!("{:02}:{:02}",m,s);
    let timer_style = Style::default()
    .fg(timer_fg(&app.state, &app.phase)).add_modifier(Modifier::BOLD);

    let time_widget = Paragraph::new(Span::styled(time_str,timer_style)
    ).alignment(Alignment::Center);
    f.render_widget(time_widget, chunks[2]);




    let status_widget = match app.state {
        TimerState::Running => {
            let msg = if app.progress() > 0.9 {
                // Increasingly frantic messages as time runs out
                match ((app.progress() - 0.9) * 100.0) as u32 {
                    0..7  => "almost there...",
                    _    => "FINISHED",
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
                " PAUSED ",
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
                    .fg(phase_color(&app.phase))
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center)
        }
    };
    f.render_widget(status_widget, chunks[4]);

    let (phase_label, phase_bg) = match app.phase {
        Phase::Work       => (" ● FOCUS ",      RED),
        Phase::ShortBreak => (" ● SHORT BREAK ", GREEN),
        Phase::LongBreak  => (" ● LONG BREAK ",  ORANGE),
    };

    let pomodoro_dots: String = {
        let done = app.pomodoros_done % 4;
        let filled = "\u{25CF}".repeat(done as usize);      // ●
        let empty  = "\u{25CB}".repeat((4 - done) as usize); // ○
        format!("{}{}", filled, empty)
    };

    let phase_widget = Line::from(vec!(
        Span::styled(
            phase_label,
            Style::default()
                .fg(BG)
                .bg(phase_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(pomodoro_dots, Style::default().fg(phase_bg)),
    )).alignment(Alignment::Center);
    
    f.render_widget(phase_widget, chunks[0]);

    let hints = match app.state {
        TimerState::Done => "[enter] next phase   [q] quit",
        _ => "[space] pause/resume   [s] skip   [r] reset   [t] settings   [q] quit",
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            hints,
            Style::default().fg(Color::Rgb(60, 60, 80)),
        ))
        .alignment(Alignment::Center),
        chunks[5],
    );
}

pub fn render(f: &mut Frame, app: &App){
    match app.screen {
        Screen::Timer => render_timer(f, app),
        Screen::Settings => render_settings(f, app),
    }
}