// main.rs
//
// The entry point. Responsible for three things:
//   1. Putting the terminal into "raw mode" and setting up ratatui
//   2. Running the event loop (tick + draw + handle input)
//   3. Tearing down the terminal cleanly on exit, even if we crash
//
// The pattern used here -- setup, loop, teardown -- is standard for
// every ratatui app. The specifics change but the skeleton stays the same.

mod app;
mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::app::{App, TimerState};

// How often we redraw. 100ms = 10fps, plenty for a timer.
// Lower this (e.g. 50ms) for smoother bar animation at the cost of more CPU.
const TICK_RATE: Duration = Duration::from_millis(100);

fn main() -> Result<()> {
    // ---- Terminal setup ----
    //
    // "Raw mode" disables line buffering and echo. Without it, keystrokes
    // wouldn't arrive until the user pressed Enter, which is useless for
    // a real-time app. crossterm handles this cross-platform.
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    // EnterAlternateScreen switches to a second terminal buffer (like vim does).
    // This means when we exit, the user's previous terminal output is restored.
    execute!(stdout, EnterAlternateScreen)?;

    // CrosstermBackend wraps stdout and knows how to write ANSI escape codes.
    // Terminal<CrosstermBackend<...>> is ratatui's top-level object.
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Hide the cursor so it doesn't flicker over our UI.
    terminal.hide_cursor()?;

    // ---- Run the app (with guaranteed cleanup) ----
    //
    // We wrap the main loop in a closure so that if it panics or returns
    // an error, the cleanup below still runs. This is important: if we crash
    // in raw mode without calling disable_raw_mode(), the user's terminal
    // will be left in a broken state where they can't see what they type.
    let result = run_app(&mut terminal);

    // ---- Terminal teardown (always runs) ----
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Propagate any error from the main loop.
    result?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();
    let mut last_tick = Instant::now();

    loop {
        // ---- Draw ----
        //
        // terminal.draw() is the core ratatui call.
        // It gives us a Frame, we pass that to our UI module which fills it,
        // then ratatui diffs the new frame against the previous one and
        // writes only the changed cells to stdout. Efficient by default.
        terminal.draw(|f| ui::render(f, &app))?;

        // ---- Event handling ----
        //
        // crossterm::event::poll() returns true if an event is available
        // within the given timeout. We pass in however much of our tick
        // interval is remaining, so we don't block longer than TICK_RATE.
        //
        // This is the standard ratatui event loop pattern:
        // poll with a timeout, handle if event exists, tick regardless.
        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            // event::read() blocks until an event arrives. Since we already
            // know one is available (poll returned true), this is instant.
            if let Event::Key(key) = event::read()? {
                // KeyEventKind::Press filters out key-repeat and release events.
                // Without this, holding a key fires multiple times on some OSes.
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),

                        KeyCode::Char(' ') => {
                            app.toggle_pause();
                        }

                        KeyCode::Enter => {
                            if app.state == TimerState::Done {
                                app.advance();
                            }
                        }

                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            app.skip();
                        }

                        _ => {}
                    }
                }
            }
        }

        // ---- Tick ----
        //
        // Once per TICK_RATE we advance the app state.
        // last_tick tracks when we last ticked; if we spent most of the
        // interval waiting for events, we still tick on schedule.
        if last_tick.elapsed() >= TICK_RATE {
            app.tick();
            last_tick = Instant::now();
        }
    }
}
