mod buffer;
mod editor;
mod highlight;
mod keymap;
mod plugin;
mod register;
mod renderer;
mod terminal;
mod types;
mod undo;

use crate::editor::Editor;
use crate::types::Keymap;
use clap::Parser;

#[derive(Parser)]
#[command(name = "nestvim")]
#[command(version = "0.1.0")]
#[command(about = "A minimal Vim-like TUI editor written in Rust", long_about = None)]
struct Cli {
    /// Keymap: vim or emacs
    keymap: Option<String>,
    /// File to edit
    file: Option<String>,
}

fn parse_keymap(s: Option<String>) -> Keymap {
    match s.as_deref() {
        Some("emacs") => Keymap::Emacs,
        _ => Keymap::Vim,
    }
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
    let keymap = parse_keymap(cli.keymap);

    let mut editor = Editor::new(cli.file.as_deref(), keymap).await?;
    editor.run().await?;

    Ok(())
}
