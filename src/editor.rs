use crate::Terminal;
use crate::Document;
use crate::Row;
use std::time::{Duration, Instant};
use std::env;
use crossterm::{
    event::{KeyCode, KeyEvent, Event,self, KeyModifiers},
    style::{Colors, Color},
};

const STATUS_BG_COLOR: Color = Color::Rgb{r: 239, g: 239, b: 239};
const STATUS_FG_COLOR: Color = Color::Rgb { r: 63, g: 63, b: 63 };
const QUIT_TIMES: u8 = 3;

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}
struct StatusMessage {
    text: String,
    time: Instant,
}
impl StatusMessage{
    fn from(message: String) -> Self {
        Self { text: message, time: Instant::now() }
    }
}

pub struct Editor {
    terminal: Terminal,
    should_quit: bool,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    quit_times: u8,
    
}

impl Editor {
    
    fn explode(e: std::io::Error){
        Terminal::clear_screen();
        panic!("{}",e);
    }
    
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-c or Esc = quit | Ctrl-s = save | Ctrl-f = find");
        let document = if args.len() > 1 {
            let file_name = &args[1];
            let doc = Document::open(&file_name);
            if let Ok(doc) = doc {
                doc
            } else {
                initial_status = format!("Error could not open file {}",file_name);
                Document::default()
            }
        }else{
            Document::default()
        };
        Self {
            
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            should_quit: false,
            cursor_position: Position::default(),
            document,
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
        }
    }
    
    fn process_keypress(&mut self) -> crossterm::Result<()> {
        let pressed_key = self.read_key()?;
        match (pressed_key.code, pressed_key.modifiers){
            (KeyCode::Char('c'),KeyModifiers::CONTROL) | (KeyCode::Esc, _) => {
                if self.quit_times > 0 && self.document.needs_saving() {
                    self.status_message = StatusMessage::from(
                        format!("WARNING! File has unsaved changes still. Press Esc {} more times to quit", self.quit_times));
                    self.quit_times -= 1;
                    return Ok(());
                }
                self.should_quit = true;
            }
            (KeyCode::Char('s'),KeyModifiers::CONTROL) => self.save(),
            (KeyCode::Char('f'),KeyModifiers::CONTROL) => {
                if let Some(query) = self.prompt("Search: ",|editor, _, query|{
                    if let Some(position) = editor.document.find(&query) {
                        editor.cursor_position = position;
                        editor.scroll();
                    }
                })
                .unwrap_or(None) {
                    if let Some(position) = self.document.find(&query[..]) {
                        self.cursor_position = position;
                    } else {
                        self.status_message = StatusMessage::from(format!("Not found :{}.",query));
                    }
                }
            }
            (KeyCode::Down, _) | (KeyCode::Up, _) | (KeyCode::Left, _) | (KeyCode::Right, _) | (KeyCode::PageDown,_) |
            (KeyCode::PageUp,_)   => self.move_cursor(pressed_key.code),
            (KeyCode::Delete,_) => self.document.delete(&self.cursor_position),
            (KeyCode::Backspace,_) => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(KeyCode::Left);
                    self.document.delete(&self.cursor_position);
                }
            }
            (KeyCode::Enter,_) => {
                self.document.insert(&self.cursor_position, '\n');
                self.move_cursor(KeyCode::Right);
            }
            (KeyCode::Char(c),_) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(KeyCode::Right);
            }
            _ => (),
        }
        self.scroll();
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
    }
    
    fn scroll(&mut self) {
        let Position {x, y} = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let offset = &mut self.offset;
        if y < offset.y {
            offset.y = y
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }
    
    fn move_cursor(&mut self, key: KeyCode) {
        let terminal_height = self.terminal.size.height as usize;
        let Position {mut y, mut x} = self.cursor_position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            KeyCode::Up => y = y.saturating_sub(1),
            KeyCode::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            KeyCode::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            KeyCode::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            KeyCode::PageUp => {
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            }
            KeyCode::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height
                }
            }
            KeyCode::Home => x = 0,
            KeyCode::End => x = width,
            _ => (),
        }
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }
        
        self.cursor_position = Position { x, y }  
    }
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen()   {
                Self::explode(error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress(){
                Self::explode(error);
            }
            
        }
    }
    fn read_key(&self) -> crossterm::Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(event) = event::read()? {
                    return Ok(event);
                }
            }
        }
    }
    fn refresh_screen(&self) ->  Result<(), std::io::Error>{
        Terminal::cursor_position(&Position::default());
        Terminal::cursor_hide();
        if self.should_quit{
            Terminal::quit()
        }
        self.draw_rows();
        self.draw_status_bar();
        self.draw_message_bar();
        Terminal::cursor_position(&Position {
            x: self.cursor_position.x.saturating_sub(self.offset.x),
            y: self.cursor_position.y.saturating_sub(self.offset.y),
        });
        Terminal::cursor_show();
        Terminal::flush()
    }
    
    fn prompt<C>(&mut self, prompt: &str, callback: C) -> std::io::Result<Option<String>> 
    where 
        C: Fn(&mut Self, Key, &String), 
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;
            let key = self.read_key()?.code; 
            match key {
                KeyCode::Backspace => {
                    if !result.is_empty() {
                        result.truncate(result.len() - 1);
                    }
                }
                KeyCode::Enter => break,
                KeyCode::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                KeyCode::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
            callback(self, key, &result);
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }
    
    fn save(&mut self) {
        if self.document.file_name.is_none(){
            let new_name = self.prompt("Save as: ",|_, _, _| {}).unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                return;
            }
            self.document.file_name = new_name;
        }
        
        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved seccessfully".to_string());
        } else {
            self.status_message = StatusMessage::from("Error while writing this file!".to_string());
        }
        
    }
    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size.width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start, end);
        println!("{}\r",row)
    }
    fn draw_rows(&self) {
        let screen_rows = self.terminal.size.height as usize;
        for terminal_row in 0..screen_rows {
            Terminal::clear_line();
            if let Some(row) = self.document.row(terminal_row + self.offset.y ) {
                self.draw_row(row);
            } else if terminal_row == screen_rows / 3 && self.document.is_empty() {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
            
        }
    }
    fn draw_welcome_message(&self) {            
        let mut welcome_message = format!("Welcome to my text editor");            
        let width = self.terminal.size.width as usize;            
        let len = welcome_message.len();            
        let padding = width.saturating_sub(len) / 2;            
        let spaces = " ".repeat(padding.saturating_sub(1));            
        welcome_message = format!("~{}{}", spaces, welcome_message);            
        welcome_message.truncate(width);            
        println!("{}\r", welcome_message);            
    }
    
    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.needs_saving() {
            " (modified)"
        } else {
            ""
        };
        
        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20)
        }
        status = format!("{} - {} lines{}",file_name, self.document.len(),modified_indicator);
        let line_indicator = format!(
            "{}/{}", self.cursor_position.y.saturating_add(1),self.document.len());
            let len = status.len() + line_indicator.len();
            if width > len {
                status.push_str(&" ".repeat(width-len));
            }
            status = format!("{}{}",status,line_indicator);
            status.truncate(width);
            Terminal::set_colors(Colors::new(STATUS_FG_COLOR, STATUS_BG_COLOR));
            println!("{}\r",status);
            Terminal::reset_colors();
        }
        
        fn draw_message_bar(&self) {
            Terminal::clear_line();
            let message = &self.status_message;
            if Instant::now() - message.time < Duration::new(5,0) {
                let mut text = message.text.clone();
                text.truncate(self.terminal.size().width as usize);
                print!("{}",text);
            }
        }
        
        
    }