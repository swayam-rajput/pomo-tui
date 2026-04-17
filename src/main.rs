mod app;
mod ui;
mod notify;

use std::{
    io,
    time::{Duration,Instant},
};
// use anyhow::Result;
use crossterm::{
    event::{self,Event,KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use notify_rust::Notification;
use ratatui::{Terminal, backend::{ CrosstermBackend}};
use anyhow::Result;
use app::App;

use crate::{app::{Screen, TimerState}, notify::send_notification};

const TICK_RATE: Duration = Duration::from_millis(100);

fn main() -> Result<()>{
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let _ =execute!(stdout,EnterAlternateScreen);

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    let _ = execute!(terminal.backend_mut(),LeaveAlternateScreen);
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
    let mut app = App::new();
    let mut last_tick = Instant::now();
    if app.should_notify(){
        Notification::new()
        .summary("focus session started")
        .body("your timer is ready").timeout(Duration::from_millis(1000))
        .show()
        .ok();
    }
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
}
