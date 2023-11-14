use crate::Row;
use crate::Position;
use std::fs;
use std::io::Write;


#[derive(Default)]
pub struct  Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    unsaved_changes: bool,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        for value in contents.lines() {
            rows.push(Row::from(value));
        }
        Ok(Self { 
            rows,
            file_name: Some(filename.to_string()),
            unsaved_changes: false,
         })
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
        let new_row = curr_row.split(at.x);
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
            self.rows.push(row);
        } else if at.y < self.len() {
            let row = &mut self.rows[at.y];
            row.insert(at.x,c);
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
        }else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
        }

    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
        }
        self.unsaved_changes = false;
        Ok(())
    }

    pub fn find(&self, query: &str) -> Option<Position> {
        for (y, row) in self.rows.iter().enumerate() {
            if let Some(x) = row.find(query) {
                return Some(Position { x, y })
            }
        }
        None
    }

    pub fn needs_saving(&self) -> bool{
        self.unsaved_changes
    }
}