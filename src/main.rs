mod app;
mod ui;

use std::{
    io,
    time::{Duration,Instant},
};
// use anyhow::Result;
use crossterm::{
    event::{self,Event,KeyCode},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::{self, CrosstermBackend}};
use anyhow::Result;
use app::App;
use std::{thread::sleep};
use notify_rust::Notification;

use crate::ui::render;

const TICK_RATE: Duration = Duration::from_millis(10);

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

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new(10);
    let mut last_tick = Instant::now();

    loop{
        terminal.draw(|f| ui::render(f, &app));
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('r') => app.reset(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= TICK_RATE{
            last_tick = Instant::now();
        }

        if app.progress() >= 1.0{
            break;
        }
    }
    Ok(())

}
