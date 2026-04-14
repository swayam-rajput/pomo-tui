use std::fmt::format;

use ratatui::{
    Frame, widgets::{Block, Borders, Gauge, Paragraph}
};
use crate::app::App;

pub fn render(f: &mut Frame, app: &App){
    let gauge = Gauge::default().block(Block::default().title("Timer").borders(Borders::ALL)).ratio(app.progress());

    f.render_widget(gauge, f.size());
    let (m,s) = app.remaining();
    let time_str = format!("{:02}:{:02}",m,s);
    // println!("{}",time_str)
    
}
