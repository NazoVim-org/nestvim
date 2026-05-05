mod buffer;
mod editor;
mod highlight;
mod plugin;
mod renderer;
mod terminal;
mod types;

use crate::editor::Editor;
use clap::Parser;

#[derive(Parser)]
#[command(name = "nestvim")]
#[command(version = "0.1.0")]
#[command(about = "A minimal Vim-like TUI editor written in Rust", long_about = None)]
struct Cli {
    /// File to edit
    file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt().try_init();
    
    let cli = Cli::parse();
    
    let mut editor = Editor::new(cli.file.as_deref()).await?;
    editor.run().await?;
    
    Ok(())
}
