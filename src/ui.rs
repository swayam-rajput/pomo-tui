
use ratatui::{
    Frame, 
    layout::{ Alignment, Layout, Direction, Constraint }, 
    style::{Color, Style}, 
    text::Span, 
    widgets::{Block, Borders, Gauge, Paragraph}
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App){

    

    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),
        Constraint::Length(3),
        // Constraint::Min(1),
    ]).split(f.area());

    let gauge = Gauge::default().block(Block::bordered().title("Timer").borders(Borders::ALL)).ratio(app.progress());

    f.render_widget(gauge, chunks[0]);
    let (m,s) = app.remaining();
    let time_str = format!("{:02}:{:02}",m,s);
    // println!("{}",time_str)
    let time_widget = Paragraph::new(
        Span::styled(
            time_str,
            Style::default().fg(Color::LightCyan),
        )
    ).alignment(Alignment::Center);
    f.render_widget(time_widget, chunks[1]);
}
