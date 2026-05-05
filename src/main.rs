mod buffer;
mod editor;
mod highlight;
mod plugin;
mod register;
mod renderer;
mod terminal;
mod types;

use crate::editor::Editor;
use clap::Parser;
use crossterm;

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
    
    // Set panic hook to restore terminal state
    std::panic::set_hook(Box::new(|_| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
    }));
    
    let cli = Cli::parse();
    
    let mut editor = Editor::new(cli.file.as_deref()).await?;
    editor.run().await?;
    
    Ok(())
}
