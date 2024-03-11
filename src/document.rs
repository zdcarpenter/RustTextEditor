use crate::FileType;
use crate::Row;
use crate::Position;
use crate::SearchDirection;
use std::fs;
use std::io::Write;


#[derive(Default)]
pub struct  Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    unsaved_changes: bool,
    file_type: FileType,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let file_type = FileType::from(filename);
        let mut rows = Vec::new();
        for value in contents.lines() {
            let mut row = Row::from(value);
            row.highlight(file_type.highlighting_options(), None);
            rows.push(row);
        }
        Ok(Self { 
            rows,
            file_name: Some(filename.to_string()),
            unsaved_changes: false,
            file_type,
         })
    }
    
    pub fn file_type(&self) -> String {
        self.file_type.name()
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn insert_newline(&mut self,at: &Position) {
        if at.y == self.len() {
            self.rows.push(Row::default());
            return;
        }
        let curr_row =  &mut self.rows[at.y];
        let mut new_row = curr_row.split(at.x);
        curr_row.highlight(self.file_type.highlighting_options(), None);
        new_row.highlight(self.file_type.highlighting_options(), None);
        self.rows.insert(at.y + 1, new_row)
        
    }

    pub fn insert(&mut self,at: &Position,c: char) {
        if at.y > self.len() {
            return;
        }
        self.unsaved_changes = true;
        if c == '\n' {
            self.insert_newline(at);
            return;
        }
        if at.y == self.len() {
            let mut row = Row::default();
            row.insert(0,c);
            row.highlight(self.file_type.highlighting_options(), None);
            self.rows.push(row);
        } else if at.y < self.len() {
            let row = &mut self.rows[at.y];
            row.insert(at.x,c);
            row.highlight(self.file_type.highlighting_options(), None);
        }
    }

    pub fn delete(&mut self, at: &Position){
        let len = self.len();
        if at.y >= len {
            return;
        }
        if at.x == self.rows[at.y].len() && at.y < len - 1 {
            let next_row = self.rows.remove(at.y+1);
            let row = &mut self.rows[at.y];
            row.append(&next_row);
            row.highlight(self.file_type.highlighting_options(), None);
        }else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
            row.highlight(self.file_type.highlighting_options(), None);
        }

    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            self.file_type = FileType::from(file_name);
            for row in &mut self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
                row.highlight(self.file_type.highlighting_options(), None)
            }
        }
        
        self.unsaved_changes = false;
        Ok(())
    }

    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        if at.y >= self.rows.len(){
            return None;
        }
        let mut position = Position {x: at.x, y: at.y};

        let start = if direction == SearchDirection::Forward {
            at.y
        } else {
            0
        };
        let end = if direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            at.y.saturating_add(1)
        };
        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(&query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }
                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }
            } else {
                return None;
            }
        }
        None
    }

    pub fn needs_saving(&self) -> bool{
        self.unsaved_changes
    }

    pub fn highlight(&mut self, word: Option<&str>) {
        for row in &mut self.rows {
            row.highlight(self.file_type.highlighting_options(), word)
        }
    }
}