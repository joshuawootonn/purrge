use configuration::get_configuration;
use globset::{Glob, GlobSetBuilder};
use walkdir::WalkDir;

use std::io::{self, stdout};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};

mod configuration;

fn main() -> io::Result<()> {
    let configuration = get_configuration();

    let mut builder = GlobSetBuilder::new();

    builder.add(Glob::new("**/node_modules").unwrap());
    builder.add(Glob::new("**/dist").unwrap());
    builder.add(Glob::new("**/.git").unwrap());
    let match_these_glob = builder.build().unwrap();

    let mut builder2 = GlobSetBuilder::new();

    builder2.add(Glob::new("**/node_modules/*").unwrap());
    builder2.add(Glob::new("**/dist/*").unwrap());
    builder2.add(Glob::new("**/.git/*").unwrap());
    let dont_match_glob = builder2.build().unwrap();

    // TODO: don't keep walking when in excluded directory or hidden directory
    let walker = WalkDir::new(configuration.directory.to_str().unwrap()).into_iter();
    let mut list = Vec::new();
    for entry in walker {
        let entry = entry.unwrap();
        let a = match_these_glob.matches(entry.path()).len();
        let b = dont_match_glob.matches(entry.path()).len();

        if a > 0 && b == 0 {
            println!("{:?}", entry.path());
            list.push(entry.path().to_path_buf());
        }
    }

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(ui)?;
        should_quit = handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn ui(frame: &mut Frame) {
    let mut state = ListState::default();
    let items = ["Item 1", "Item 2", "Item 3"];
    let list = List::new(items)
        .block(Block::default().title("List").borders(Borders::ALL))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true);

    frame.render_stateful_widget(list, frame.size(), &mut state);
    // frame.render_widget(
    //     Paragraph::new("Hello World!")
    //         .block(Block::default().title("Greeting").borders(Borders::ALL)),
    //     frame.size(),
    // );
}
