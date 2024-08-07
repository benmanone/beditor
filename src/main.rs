use crate::editor::Editor;

mod buffer;
mod editor;
mod terminal;
mod view;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if let Err(err) = Editor::new(&args.get(1).cloned()).run() {
        println!("FATAL: {err}");
    }
}
