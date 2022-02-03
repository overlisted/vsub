#![feature(string_remove_matches)]
#![feature(pattern)]

mod commands;
mod line_table;
mod session;
mod widgets;

use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::{event, terminal};
use std::{env, error, fs, io, panic, process};
use tui::layout::Rect;

fn on_panic(info: &panic::PanicInfo) {
    crossterm::execute!(io::stderr(), terminal::LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();

    println!("{}", info);
    println!("{:?}", backtrace::Backtrace::new());
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut args = env::args().skip(1);

    let file = if let Some(x) = args.next() {
        x
    } else {
        eprintln!("Usage: vsub <filename>");
        process::exit(1);
    };

    let file = fs::OpenOptions::new().read(true).write(true).open(file)?;

    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, terminal::EnterAlternateScreen)?;

    panic::set_hook(Box::new(on_panic));

    let mut terminal = tui::Terminal::new(tui::backend::CrosstermBackend::new(io::stdout()))?;
    let mut session = session::Session::new(file)?;

    loop {
        terminal.draw(|f| session.ui(f))?;

        let event = event::read()?;

        match event {
            event::Event::Resize(w, h) => terminal.resize(Rect::new(0, 0, w, h))?,
            event::Event::Key(event::KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }) => break,
            event::Event::Key(e) => session.key(e),
            _ => {}
        }
    }

    crossterm::execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
