pub struct FileType {
    name: String,
    hl_options: HighlightingOptions,
}

#[derive(Default, Clone, Copy)]
pub struct HighlightingOptions {
    numbers: bool,
    strings: bool,
    characters: bool,
    comments: bool,
}

impl HighlightingOptions {
    pub fn numbers(self) -> bool {
        self.numbers
    }

    pub fn strings(self) -> bool {
        self.strings
    }

    pub fn characters(self) -> bool {
        self.characters
    }

    pub fn comments(self) -> bool {
        self.comments
    }
}
impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("No filetype"),
            hl_options: HighlightingOptions::default(),
        }
    }
}

impl FileType {
    pub fn name(&self) -> String{
        self.name.clone()
    }

    pub fn from(file_name: &str) -> Self {
        if file_name.ends_with(".rs") {
            return Self {
                name: String::from("Rust"),
                hl_options: HighlightingOptions { numbers: true, strings: true, characters: true, comments: true, },
            };
        }
        Self::default()
    }

    pub fn highlighting_options(&self) -> HighlightingOptions {
        self.hl_options
    }
}