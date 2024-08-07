use crate::editor::Mode;
use crossterm::cursor;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::Command;
use std::fmt::Display;
use std::io::stdout;
use std::io::Error;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn left(&mut self) {
        if self.x != 0 {
            self.x -= 1;
        }
    }

    pub fn right(&mut self) {
        if self.x != terminal::size().unwrap().0 {
            self.x += 1;
        }
    }

    pub fn up(&mut self) {
        if self.y > 0 {
            self.y -= 1;
        }
    }

    pub fn down(&mut self) {
        if self.y != terminal::size().unwrap().1 - 2 {
            self.y += 1;
        }
    }
}

impl Position {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

pub fn initialise() -> Result<(), Error> {
    enable_raw_mode()?;
    clear_screen()?;
    move_cursor_to(&Position::new(0, 0))?;
    execute()?;
    Ok(())
}

pub fn terminate() -> Result<(), Error> {
    execute()?;
    disable_raw_mode()?;
    Ok(())
}

pub fn clear_screen() -> Result<(), Error> {
    queue_command(Clear(ClearType::All))?;
    Ok(())
}

pub fn clear_line() -> Result<(), Error> {
    queue_command(Clear(ClearType::CurrentLine))?;
    Ok(())
}

pub fn print(str: impl Display) -> Result<(), Error> {
    queue_command(Print(str))?;
    Ok(())
}

pub fn queue_command(command: impl Command) -> Result<(), Error> {
    queue!(stdout(), command)?;
    Ok(())
}

pub fn execute() -> Result<(), Error> {
    stdout().flush()
}

pub fn move_cursor_to(pos: &Position) -> Result<(), Error> {
    queue_command(cursor::MoveTo(pos.x, pos.y))?;
    Ok(())
}

pub fn hide_cursor() -> Result<(), Error> {
    queue_command(cursor::Hide)
}

pub fn show_cursor() -> Result<(), Error> {
    queue_command(cursor::Show)
}

pub fn change_cursor_style(mode: &Mode) {
    match mode {
        Mode::Normal => queue_command(cursor::SetCursorStyle::SteadyBlock).unwrap(),
        Mode::Insert => queue_command(cursor::SetCursorStyle::BlinkingBar).unwrap(),
    }
}
