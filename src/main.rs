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
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nestvim")]
#[command(version = "0.1.0")]
#[command(about = "A minimal Vim-like TUI editor written in Rust", long_about = None)]
struct Cli {
    /// Mode command: vim or emacs
    #[command(subcommand)]
    mode: Option<ModeCommand>,

    /// File to edit when no mode command is specified
    file: Option<String>,
}

#[derive(Subcommand)]
enum ModeCommand {
    /// Start in Vim keymap mode
    Vim {
        /// File to edit
        file: Option<String>,
    },
    /// Start in Emacs keymap mode
    Emacs {
        /// File to edit
        file: Option<String>,
    },
}

fn resolve_cli(cli: Cli) -> (Keymap, Option<String>) {
    match cli.mode {
        Some(ModeCommand::Vim { file }) => (Keymap::Vim, file),
        Some(ModeCommand::Emacs { file }) => (Keymap::Emacs, file),
        None => (Keymap::Vim, cli.file),
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
    let (keymap, file) = resolve_cli(cli);

    let mut editor = Editor::new(file.as_deref(), keymap).await?;
    editor.run().await?;

    Ok(())
}
