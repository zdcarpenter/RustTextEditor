use std::io::{stdout, Write};
use crossterm::{
    cursor,
    terminal::{ClearType, self},
    execute, QueueableCommand,
    style::{Colors, SetColors, ResetColor},
};

use crate::editor::Position;
pub struct Size {
    pub width: u16,
    pub height: u16,
}
pub struct Terminal {
    pub size: Size,
}


impl Terminal {

    pub fn default() -> Result<Terminal, std::io::Error>{
        let size = terminal::size().unwrap();
        terminal::enable_raw_mode().expect("Could not turn on Raw mode");
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            }
        })
    }

    pub fn clear_screen() {
        execute!(stdout(), terminal::Clear(ClearType::All)).ok();
    }

    pub fn clear_line() {
        execute!(stdout(), terminal::Clear(ClearType::CurrentLine)).ok();
    }

    pub fn quit() {
        Terminal::clear_screen();
        crossterm::terminal::disable_raw_mode().ok();
        println!("bye \r");
    }

    pub fn size(&self) -> &Size{
        &self.size
    }
    pub fn cursor_position(position: &Position) {
        let Position { mut x, mut y } = position;
        x = x.saturating_add(1);
        y = y.saturating_add(1);
        let x = x as u16;
        let y = y as u16;

        stdout().queue(cursor::MoveTo(x - 1, y - 1)).ok();
    }

    pub fn cursor_hide() {
        execute!(stdout(), cursor::DisableBlinking).ok();
    }

    pub fn cursor_show() {
        execute!(stdout(), cursor::DisableBlinking).ok();
    }

    pub fn flush() -> Result<(),std::io::Error>{
        stdout().flush()
    }

    pub fn set_colors(colors: Colors) {
        execute!(stdout(), SetColors(colors)).ok();
    }
    
    pub fn reset_colors() {
        execute!(stdout(), ResetColor).ok();
    }
    
}