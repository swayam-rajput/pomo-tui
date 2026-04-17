mod app;
mod ui;

use std::{
    io,
    time::{Duration,Instant},
};
// use anyhow::Result;
use crossterm::{
    event::{self,Event,KeyCode, KeyEventKind},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::{self, CrosstermBackend}};
use anyhow::Result;
use app::App;
use std::{thread::sleep};
use notify_rust::Notification;

use crate::app::{Screen, TimerState};
use crate::ui::render;

const TICK_RATE: Duration = Duration::from_millis(100);

fn main() -> Result<()>{
    let app = App::new(4);
    // Notification::new()
    // .summary("Pomodoro Started")
    // .auto_icon()
    // .body("Focus and get to work.")
    // .show()
    // .unwrap();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout,EnterAlternateScreen);

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(),LeaveAlternateScreen);
    terminal.show_cursor()?;
    result
}


fn handle_timer_keys(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Char('q') => return true, // signal quit
        KeyCode::Char(' ') => app.toggle_pause(),
        KeyCode::Char('r') => app.reset(),
        KeyCode::Char('s') => app.skip(),
        KeyCode::Char('t') => app.screen = Screen::Settings,
        KeyCode::Enter => {
            if app.state == TimerState::Done {
                app.advance();
            }
        }
        _ => {}
    }
    false
}
fn handle_settings_keys(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Up => app.settings_up(),
        KeyCode::Down => app.settings_down(),
        KeyCode::Left => app.adjust_selected(-1),
        KeyCode::Right => app.adjust_selected(1),
        KeyCode::Char('H') => app.adjust_selected(-5),
        KeyCode::Char('L') => app.adjust_selected(5),
        KeyCode::Char('t') | KeyCode::Enter => app.screen = Screen::Timer,
        KeyCode::Char('q') => return true,
        _ => {}
    }
    false
}





fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new(30);
    let mut last_tick = Instant::now();

    loop{
        terminal.draw(|f| ui::render(f, &app))?;
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press{
                    let should_quit = match app.screen {
                        Screen::Timer => handle_timer_keys(&mut app, key.code),
                        Screen::Settings => handle_settings_keys(&mut app, key.code),
                    };

                    if should_quit {
                        return Ok(());
                    }
                }
            }
        }

        if last_tick.elapsed() >= TICK_RATE {
            app.tick();
            last_tick = Instant::now();
        }


    }
    Ok(())

}
