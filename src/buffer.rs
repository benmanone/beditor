use crate::terminal::Position;
use std::{fs::File, io::Write};

pub struct Buffer {
    pub lines: Vec<String>,
    pub history: History,
    pub file: String,
}

impl Buffer {
    pub fn new(lines: Vec<String>, file: String) -> Self {
        Self {
            lines: lines.clone(),
            file,
            history: History {
                states: vec![lines],
                cursors: vec![Position::new(0, 0)],
                index: 0,
            },
        }
    }

    pub fn len(&self) -> u16 {
        self.lines
            .len()
            .try_into()
            .expect("Couldn't cast length...")
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn write(&mut self, pos: &Position, char: char) {
        let len: u16 = self.lines.len().try_into().unwrap();

        if pos.y < len {
            let split = self
                .lines
                .get(pos.y as usize)
                .unwrap()
                .split_at(pos.x as usize);

            let line = format!("{}{}{}", split.0, char, split.1);
            let mut split_lines = <[&[String]; 2]>::from(self.lines.split_at(pos.y as usize));
            split_lines[1] = &split_lines[1][1..];

            self.lines = [
                split_lines.first().unwrap(),
                vec![line].as_slice(),
                split_lines.last().unwrap(),
            ]
            .concat();
        } else {
            // Add lines to end of document
            let difference: u16 = pos.y - len;

            if difference > 0 {
                for _ in 0..difference {
                    self.lines.push(String::new());
                }
            }
            self.lines.push(String::from(char));
        }
    }

    pub fn backspace(&mut self, pos: &Position) -> Backspace {
        if pos.y < self.lines.len().try_into().unwrap() && pos.x > 0 {
            let split = self
                .lines
                .get(pos.y as usize)
                .unwrap()
                .split_at(pos.x as usize);

            let line = format!(
                "{}{}",
                split.0.strip_suffix(|_: char| true).unwrap(),
                split.1
            );
            let mut split_lines = <[&[String]; 2]>::from(self.lines.split_at(pos.y as usize));
            split_lines[1] = &split_lines[1][1..];

            self.lines = [
                split_lines.first().unwrap(),
                vec![line].as_slice(),
                split_lines.last().unwrap(),
            ]
            .concat();
            return Backspace::SameLine;
        } else if pos.x <= 1 && pos.y < self.lines.len().try_into().unwrap() {
            let mut split = self.lines.split_at(pos.y as usize);

            if self.nth_line_len(pos.y.into()) == 0 && pos.y > 0 {
                // Delete a line
                split.1 = &split.1[1..];

                self.lines = <[&[String]; 2]>::from(split).concat();

                return Backspace::WrapLines(Position::new(
                    self.nth_line_len((pos.y - 1) as usize),
                    pos.y - 1,
                ));
            } else if pos.y > 0 {
                // Wrap line onto line above
                let wrapped_line_len = self.nth_line_len(pos.y.into());
                self.lines = [
                    &split.0[..split.0.len() - 1],
                    vec![split.0.last().unwrap().to_owned() + split.1.first().unwrap()].as_slice(),
                    &split.1[1..],
                ]
                .concat();
                return Backspace::WrapLines(Position::new(
                    self.nth_line_len((pos.y - 1) as usize) - wrapped_line_len,
                    pos.y - 1,
                ));
            };
        }
        Backspace::SameLine
    }

    pub fn enter(&mut self, pos: &Position) {
        if pos.y >= self.lines.len().try_into().unwrap() {
            self.new_line(pos);
        } else if pos.x
            < self
                .lines
                .get(pos.y as usize)
                .unwrap()
                .len()
                .try_into()
                .unwrap()
        {
            let split = self.lines.split_at(pos.y as usize);
            let split_lines = split.1.first().unwrap().split_at(pos.x as usize);
            self.lines = [
                split.0,
                vec![split_lines.0.to_string()].as_slice(),
                vec![split_lines.1.to_string()].as_slice(),
                &split.1[1..],
            ]
            .concat();
        }
    }

    pub fn new_line(&mut self, pos: &Position) {
        let len = self.lines.len().try_into().unwrap();

        if pos.y < len {
            let split = self.lines.split_at(pos.y as usize);
            self.lines = [split.0, vec![String::new()].as_slice(), split.1].concat();
        } else {
            for _ in 0..=(pos.y - len) {
                self.lines.push(String::new());
            }
        }
    }

    pub fn nth_line_len(&self, n: usize) -> u16 {
        self.lines
            .get(n)
            .map_or(0, |str| str.len().try_into().unwrap())
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        let mut file = File::create(self.file.clone())?;
        File::set_len(&file, 0)?;

        for line in &self.lines {
            writeln!(file, "{line}")?;
        }
        Ok(())
    }

    pub fn update_history(&mut self, cursor: Position) {
        if self.history.is_in_past() {
            self.history.decapitate();
        }
        self.history.update(self.lines.clone(), cursor);
    }

    pub fn undo(&mut self) -> Option<Position> {
        if self.history.index > 0 {
            self.history.rollback();
            self.lines.clone_from(
                self.history
                    .states
                    .get(self.history.index as usize)
                    .unwrap(),
            );
            return Some(
                self.history
                    .cursors
                    .get(self.history.index as usize)
                    .unwrap()
                    .clone(),
            );
        }
        None
    }

    pub fn redo(&mut self) -> Option<Position> {
        if self.history.is_in_past() {
            self.history.rollforward();
            self.lines.clone_from(
                self.history
                    .states
                    .get(self.history.index as usize)
                    .unwrap(),
            );
            return Some(
                self.history
                    .cursors
                    .get(self.history.index as usize)
                    .unwrap()
                    .clone(),
            );
        }
        None
    }
}

pub enum Backspace {
    WrapLines(Position),
    SameLine,
}

pub struct History {
    pub states: Vec<Vec<String>>,
    pub cursors: Vec<Position>,
    pub index: u32,
}

impl History {
    pub fn update(&mut self, new: Vec<String>, cursor: Position) {
        self.states.push(new);
        self.cursors.push(cursor);
        self.index += 1;
    }

    pub fn decapitate(&mut self) {
        self.states.truncate(self.index.try_into().unwrap());
        self.cursors.truncate(self.index.try_into().unwrap());
    }

    pub fn is_in_past(&self) -> bool {
        self.states.len() - 1 > self.index.try_into().unwrap()
    }

    pub fn rollback(&mut self) {
        self.index -= 1;
    }

    pub fn rollforward(&mut self) {
        self.index += 1;
    }
}
