use crate::buffer::Backspace;
use crate::view::Size;
use crate::view::View;
use crossterm::event::Event::Key;
use crossterm::event::KeyCode::Char;
use crossterm::event::{read, KeyModifiers};
use crossterm::event::{Event, KeyCode};

use crate::terminal::{
    change_cursor_style, clear_screen, execute, hide_cursor, initialise, move_cursor_to,
    show_cursor, terminate, Position,
};

#[derive(PartialEq, Eq)]
pub enum Mode {
    Insert,
    Normal,
    // Command,
}

pub struct Editor {
    pub view: View,
    pub cursor: Cursor,
    pub mode: Mode,
    pub quit: bool,
}

impl Editor {
    pub fn new(file: &Option<String>) -> Self {
        Self {
            view: View::new(file),
            cursor: Cursor::new(Position::new(0, 0)),
            mode: Mode::Normal,
            quit: false,
        }
    }

    pub fn run(&mut self) -> Result<(), std::io::Error> {
        // Allow reading of bytes directly from stdin without pressing enter
        initialise()?;

        self.repl()?;

        terminate()?;
        Ok(())
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        loop {
            self.refresh_screen()?;
            self.view.redraw = true;

            if self.quit {
                break;
            }

            let event = read()?;
            self.evaluate_event(&event)?;
        }

        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) -> Result<(), std::io::Error> {
        match event {
            Key(key) => match key.code {
                Char('q') if key.modifiers == KeyModifiers::CONTROL => {
                    self.quit = true;
                }
                Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                    self.view.save()?;
                }
                Char('h') if self.mode != Mode::Insert => self.left(),
                Char('j') if self.mode != Mode::Insert => self.down(),
                Char('k') if self.mode != Mode::Insert => self.up(),
                Char('u') if self.mode != Mode::Insert => {
                    if let Some(pos) = self.view.undo() {
                        self.cursor.position = pos;
                        self.correct_cursor();
                    }
                }
                Char('U') if self.mode != Mode::Insert => {
                    if let Some(pos) = self.view.redo() {
                        self.cursor.position = pos;
                        self.correct_cursor();
                    }
                }
                Char('l') if self.mode != Mode::Insert => self.right(),
                Char('i') if self.mode == Mode::Normal => self.mode(Mode::Insert),
                Char('a') if self.mode == Mode::Normal => {
                    self.mode(Mode::Insert);
                    self.right();
                }
                Char('I') if self.mode == Mode::Normal => {
                    self.cursor.position.x = 0;
                    self.mode(Mode::Insert);
                }
                Char('A') if self.mode == Mode::Normal => {
                    self.cursor.position.x = self.current_line_len();
                    self.mode(Mode::Insert);
                }
                Char('o') if self.mode == Mode::Normal => {
                    self.mode(Mode::Insert);
                    self.down();
                    self.view.new_line(&self.cursor.position);
                    self.correct_cursor();
                }
                KeyCode::Esc => {
                    self.mode(Mode::Normal);
                    self.view.update_history(self.cursor.position.clone());
                }
                KeyCode::Enter if self.mode == Mode::Insert => {
                    self.view.enter(&self.cursor.position);
                    self.down();
                    self.cursor.position.x = 0;
                }
                KeyCode::Backspace if self.mode == Mode::Insert => {
                    if let Backspace::WrapLines(pos) = self.view.backspace(&self.cursor.position) {
                        self.cursor.position = pos;
                    } else {
                        self.left();
                    }
                }
                KeyCode::Tab => {
                    for _ in 0..4 {
                        self.view.write(&self.cursor.position, ' ');
                        self.cursor.position.right();
                    }
                }
                Char(c) if self.mode == Mode::Insert => {
                    self.view.write(&self.cursor.position, c);
                    self.cursor.position.right();
                    self.view.draw_bottom_message("")?;
                }
                _ => (),
            },
            Event::Resize(x, y) => self.view.resize(Size {
                width: *x,
                height: *y,
            }),
            _ => (),
        }

        move_cursor_to(&self.cursor.position)?;
        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        hide_cursor()?;

        if self.quit {
            clear_screen()?;
            print!("Goodbye.");
        } else {
            self.view.render(&self.cursor.position)?;
        }

        show_cursor()?;
        execute()?;

        Ok(())
    }

    fn mode(&mut self, mode: Mode) {
        self.mode = mode;
        change_cursor_style(&self.mode);
    }

    fn left(&mut self) {
        self.cursor.position.left();
        self.cursor.update();
    }

    fn right(&mut self) {
        if self.cursor.position.x < self.current_line_len() {
            self.cursor.position.right();
            self.cursor.update();
        }
    }

    fn up(&mut self) {
        self.cursor.position.up();
        if self.cursor.position.x > self.current_line_len() {
            self.correct_cursor();
        }
        self.recall_cursor();
    }

    fn down(&mut self) {
        self.cursor.position.down();
        if self.cursor.position.x > self.current_line_len() {
            // If the new position would be longer than the line, move back to the end of the line
            self.correct_cursor();
        }
        self.recall_cursor();
    }

    fn correct_cursor(&mut self) {
        self.cursor.position.x = self.current_line_len();
    }

    fn recall_cursor(&mut self) {
        let current_len = self.current_line_len();

        if self.cursor.position.x < self.cursor.previous_x {
            self.cursor.position.x = self.cursor.previous_x;
        }
        if current_len < self.cursor.previous_x {
            self.correct_cursor();
        }
    }

    fn current_line_len(&self) -> u16 {
        self.view.nth_line_len(self.cursor.position.y as usize)
    }
}

pub struct Cursor {
    position: Position,
    previous_x: u16,
}

impl Cursor {
    pub const fn new(pos: Position) -> Self {
        Self {
            position: pos,
            previous_x: 1,
        }
    }

    pub fn update(&mut self) {
        if self.position.x != self.previous_x {
            self.previous_x = self.position.clone().x;
        }
    }
}

// pub struct Command {
//     iterator: u32,
//     movement: Movement,
//     action: Action,
// }
