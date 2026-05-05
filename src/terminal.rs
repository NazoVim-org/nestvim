use atty;
use crossterm::{
    cursor::{MoveTo, Show},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, size},
};
use std::io::{self, Write, stdout};

pub struct Terminal {
    rows: u16,
    cols: u16,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        match size() {
            Ok((cols, rows)) => Ok(Self { rows, cols }),
            Err(e) => {
                eprintln!("Warning: Could not get terminal size: {}. Using defaults.", e);
                Ok(Self { rows: 24, cols: 80 })
            }
        }
    }

    pub fn enable_raw_mode(&mut self) -> io::Result<()> {
        if !atty::is(atty::Stream::Stdin) {
            return Err(io::Error::new(io::ErrorKind::Other, "Not a terminal"));
        }
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, Show)?;
        self.update_size();
        Ok(())
    }

    pub fn disable_raw_mode(&self) -> io::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, LeaveAlternateScreen)?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn update_size(&mut self) {
        if let Ok((cols, rows)) = size() {
            self.cols = cols;
            self.rows = rows;
        }
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }

    pub fn cols(&self) -> u16 {
        self.cols
    }

    pub fn move_cursor(&self, row: u16, col: u16) -> io::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, MoveTo(col.saturating_sub(1), row.saturating_sub(1)))
    }

    pub fn clear_screen(&self) -> io::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, Clear(ClearType::All))
    }

    pub fn write_line(&self, row: u16, content: &str) -> io::Result<()> {
        let mut stdout = stdout();
        let cols = self.cols as usize;
        
        // Strip ANSI escape codes for width calculation
        let visible: String = content.chars()
            .filter(|c| *c != '\x1b')
            .collect();
        
        let visible_width = visible.chars().count();
        let padded = if visible_width < cols {
            format!("{}{}", content, " ".repeat(cols - visible_width))
        } else {
            content.chars().take(cols).collect()
        };
        
        execute!(stdout, MoveTo(0, row.saturating_sub(1)))?;
        stdout.write_all(padded.as_bytes())?;
        stdout.flush()
    }

    pub fn write_status(&self, content: &str) -> io::Result<()> {
        let row = self.rows.saturating_sub(1);
        self.write_line(row, content)
    }

    pub fn flush(&self) -> io::Result<()> {
        stdout().flush()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.disable_raw_mode();
    }
}
