#![warn(clippy::all, clippy::pedantic, clippy::restriction)]
#![allow(            
    clippy::missing_docs_in_private_items,            
    clippy::implicit_return,            
    clippy::shadow_reuse,            
    clippy::print_stdout,            
    clippy::wildcard_enum_match_arm,            
    clippy::else_if_without_else            
)]
mod editor;
mod terminal;
mod document;
mod row;
mod highlighting;
mod filetype;
pub use terminal::Terminal;
pub use editor::Position;
pub use editor::SearchDirection;
pub use document::Document;
pub use row::Row; 
pub use filetype::FileType;
pub use filetype::HighlightingOptions;
use editor::Editor;




fn main() {
    Editor::default().run();
}
