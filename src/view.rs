use crate::buffer::{Backspace, Buffer};
use crossterm::terminal;
use std::fmt::Display;
use std::io::Error;

use crate::terminal::{clear_line, move_cursor_to, print, Position};

pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub fn default() -> Self {
        Self {
            width: terminal::size().expect("Couldn't get size.").0,
            height: terminal::size().expect("Couldn't get size.").1,
        }
    }
}

pub struct View {
    buffer: Buffer,
    size: Size,
    pub redraw: bool,
}

impl View {
    pub fn new(file: &Option<String>) -> Self {
        file.as_ref().map_or_else(
            || Self {
                buffer: Buffer::new(vec![String::new()], String::from("new.txt")),
                redraw: true,
                size: Size::default(),
            },
            |f| Self {
                buffer: Buffer::new(
                    std::fs::read_to_string(f)
                        .expect("FATAL: Failed to read file")
                        .lines()
                        .collect::<Vec<_>>()
                        .into_iter()
                        .map(std::string::ToString::to_string)
                        .collect(),
                    f.to_string(),
                ),
                redraw: true,
                size: Size::default(),
            },
        )
    }

    pub fn render(&self, pos: &Position) -> Result<(), Error> {
        if self.redraw {
            for row in 0..self.size.height - 1 {
                move_cursor_to(&Position::new(0, row))?;
                clear_line()?;

                if row >= self.buffer.len() {
                    print("~")?;
                } else {
                    print(
                        self.buffer
                            .lines
                            .get(row as usize)
                            .expect("FATAL: Couldn't get line"),
                    )?;
                }

                if self.buffer.is_empty() && row == terminal::size()?.1 / 3 {
                    self.welcome_message(
                        (env!("CARGO_PKG_NAME").to_uppercase() + " " + env!("CARGO_PKG_VERSION"))
                            .as_str(),
                        row,
                    )?;
                }
            }
            move_cursor_to(pos)?;
        }

        Ok(())
    }

    fn welcome_message(&self, message: &str, row: u16) -> Result<(), Error> {
        let try_message_start = self
            .size
            .width
            .checked_sub(message.len().try_into().unwrap());

        if let Some(message_start) = try_message_start {
            if self.size.width >= message.len().try_into().unwrap() {
                let start_pos = Position::new(message_start / 2, row);

                move_cursor_to(&start_pos)?;
                print(message)?;
            } else {
                move_cursor_to(&Position::new(row, 0))?;
                print(message)?;
            }
        }

        Ok(())
    }

    pub fn resize(&mut self, into: Size) {
        self.size = into;
        self.redraw = true;
    }

    pub fn write(&mut self, pos: &Position, char: char) {
        self.buffer.write(pos, char);
        self.redraw = true;
    }

    pub fn backspace(&mut self, pos: &Position) -> Backspace {
        self.redraw = true;
        self.buffer.backspace(pos)
    }

    pub fn new_line(&mut self, pos: &Position) {
        self.redraw = true;
        self.buffer.new_line(pos);
    }

    pub fn enter(&mut self, pos: &Position) {
        self.redraw = true;
        self.buffer.enter(pos);
    }

    pub fn update_history(&mut self, pos: Position) {
        self.redraw = true;
        self.buffer.update_history(pos);
    }

    pub fn undo(&mut self) -> Option<Position> {
        self.redraw = true;
        self.buffer.undo()
    }

    pub fn redo(&mut self) -> Option<Position> {
        self.redraw = true;
        self.buffer.redo()
    }

    pub fn nth_line_len(&self, n: usize) -> u16 {
        self.buffer.nth_line_len(n)
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        self.buffer.save()?;
        self.draw_bottom_message(format!("Successfully saved to {}.", self.buffer.file))?;
        Ok(())
    }

    pub fn draw_bottom_message(&mut self, message: impl Display) -> Result<(), std::io::Error> {
        move_cursor_to(&Position::new(0, terminal::size().unwrap().1))?;
        clear_line()?;
        print(message)?;
        self.redraw = true;
        Ok(())
    }
}
