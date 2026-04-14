// ─── mod declarations ─────────────────────────────────────────────────────────
// These two lines tell Rust: "there are two other files that are part of this
// program." Without them, app.rs and ui.rs would be completely ignored.
mod app;
mod ui;

// ─── Standard library imports ─────────────────────────────────────────────────
use std::{
    error::Error, // A generic trait for all error types
    io,           // Input/Output (we need this for the terminal)
    time::Duration,
};

// ─── External crate imports ───────────────────────────────────────────────────
// `crossterm` handles raw keyboard input and terminal control sequences.
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// `ratatui` handles all the drawing on screen.
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::App;

// ─── MAIN ────────────────────────────────────────────────────────────────────
// `Result<(), Box<dyn Error>>` means:
//   • Ok(()) → the program ran and exited normally
//   • Err(e)  → something went wrong; `e` is any error type (that's what `dyn Error` means)
// The `?` operator is used throughout: if a function returns Err, it immediately
// bubbles that error up to this return type instead of crashing.
fn main() -> Result<(), Box<dyn Error>> {
    // ── STEP 1: Put the terminal in "raw mode" ───────────────────────────
    // Normally, the terminal buffers your input until you press Enter.
    // Raw mode lets us receive EVERY key press instantly, one at a time.
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    // EnterAlternateScreen saves the current terminal content and gives us
    // a blank canvas. When we exit, the original content is restored.
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Build our ratatui Terminal, which wraps crossterm's backend.
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ── STEP 2: Create our App state ────────────────────────────────────
    let mut app = App::new();

    // ── STEP 3: Run the event loop ───────────────────────────────────────
    // We run the app, and save the result (Ok or Err) so we can restore
    // the terminal REGARDLESS of whether it crashed.
    let result = run(&mut terminal, &mut app);

    // ── STEP 4: ALWAYS restore the terminal ──────────────────────────────
    // This MUST happen even if the app crashed. If we skip this, the user's
    // terminal will be stuck in raw mode and look broken after we exit.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Now it's safe to surface any error that happened during the run.
    if let Err(e) = result {
        eprintln!("Error: {e:?}");
    }

    Ok(())
}

// ─── THE EVENT LOOP ──────────────────────────────────────────────────────────
// This is the heart of any TUI: draw → wait for input → update → repeat.
//
// We use a concrete type `CrosstermBackend<io::Stdout>` to avoid complex
// generic lifetime problems (keeping things beginner-friendly!).
fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    // How often we refresh the screen (100ms = 10 frames per second).
    let tick_rate = Duration::from_millis(100);

    loop {
        // ── DRAW ─────────────────────────────────────────────────────────
        // `draw` calls our `ui::render` function with a Frame to draw into.
        // The closure `|f|` captures the frame and passes it along.
        terminal.draw(|f| ui::render(f, app))?;

        // ── POLL FOR INPUT ────────────────────────────────────────────────
        // `poll` blocks for up to `tick_rate` waiting for an event.
        // If a key is pressed sooner, it returns true immediately.
        // If nothing happens in 100ms, it returns false and we just tick.
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                // Ignore key-release events on Windows (which fires both Press and Release).
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Handle key presses:
                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Char(' ') => {
                        // Space: play or pause the timer.
                        app.toggle();
                    }
                    KeyCode::Char('r') => {
                        // R: reset the current session.
                        app.reset();
                    }
                    KeyCode::Char('s') => {
                        // S: skip to the next session.
                        app.skip();
                    }
                    _ => {} // Ignore all other keys.
                }
            }
        }

        // ── TICK ──────────────────────────────────────────────────────────
        // Update the timer. This is safe to call even when paused because
        // `tick()` checks `is_running` internally and does nothing if false.
        app.tick();

        // ── CHECK EXIT ────────────────────────────────────────────────────
        if app.should_quit {
            return Ok(());
        }
    }
}