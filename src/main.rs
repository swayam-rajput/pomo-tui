mod app;
mod ui;

use std::{error::Error, io, time::Duration};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::{App, Screen};

fn main() -> Result<(), Box<dyn Error>> {
    // Put the terminal in raw mode → every keypress is delivered instantly.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    // EnterAlternateScreen → blank canvas; terminal is restored when we leave.
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let result = run(&mut terminal, &mut app);

    // ALWAYS restore the terminal — even if the app crashed.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e:?}");
    }

    Ok(())
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    // 50ms tick = 20 frames per second → visibly smooth progress bar.
    let tick_rate = Duration::from_millis(50);

    loop {
        // ── Draw ─────────────────────────────────────────────────────────
        terminal.draw(|f| ui::render(f, app))?;

        // ── Poll for keypresses ──────────────────────────────────────────
        // Waits up to `tick_rate` for an event. Returns immediately if a
        // key was pressed, so the UI reacts instantly.
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                // On Windows, key events fire for both Press and Release.
                // We only want to act on the actual Press.
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // ── Dispatch keys based on which screen is active ────────
                match app.screen {
                    Screen::Timer => handle_timer_keys(app, key.code),
                    Screen::Settings => handle_settings_keys(app, key.code),
                }
            }
        }

        // ── Tick ─────────────────────────────────────────────────────────
        // Checks if the timer finished and advances the mode if so.
        app.tick();

        if app.should_quit {
            return Ok(());
        }
    }
}

// ─── KEY HANDLERS ────────────────────────────────────────────────────────────
// Splitting key handling per-screen keeps each match arm short and clear.

fn handle_timer_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char(' ') => app.toggle(),
        KeyCode::Char('r') => app.reset(),
        // 'n' for next — skip to the next session
        KeyCode::Char('n') => app.skip(),
        // 't' opens the settings screen
        KeyCode::Char('t') => app.screen = Screen::Settings,
        _ => {}
    }
}

fn handle_settings_keys(app: &mut App, key: KeyCode) {
    match key {
        // Navigate rows
        KeyCode::Up => app.settings_up(),
        KeyCode::Down => app.settings_down(),
        // Adjust by 1 minute
        KeyCode::Left => app.adjust_selected(-1),
        KeyCode::Right => app.adjust_selected(1),
        // Larger jumps: Shift+Arrow = 5 minutes
        KeyCode::Char('H') => app.adjust_selected(-5),
        KeyCode::Char('L') => app.adjust_selected(5),
        // Go back to the timer with 't' OR Enter
        KeyCode::Char('t') | KeyCode::Enter => app.screen = Screen::Timer,
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}