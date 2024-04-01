use configuration::get_configuration;
use globset::{Glob, GlobSetBuilder};
use walkdir::WalkDir;

use std::{
    io::{self, stdout},
    path::PathBuf,
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};

mod configuration;

enum Status {
    Initial,
    InProgress,
    Deleted,
}

struct DirectoryItem {
    path: PathBuf,
    status: Status,
}

struct StatefulList {
    state: ListState,
    items: Vec<DirectoryItem>,
    last_selected: Option<usize>,
}

struct App {
    items: StatefulList,
}

impl App {
    fn new() -> App {
        App {
            items: StatefulList {
                state: ListState::default(),
                items: vec![],
                last_selected: None,
            },
        }
    }

    fn find_directory_items(&mut self, path: PathBuf) {
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
        let walker = WalkDir::new(path.to_str().unwrap()).into_iter();
        for entry in walker {
            let entry = entry.unwrap();
            let a = match_these_glob.matches(entry.path()).len();
            let b = dont_match_glob.matches(entry.path()).len();

            if a > 0 && b == 0 {
                println!("{:?}", entry.path());

                self.items.items.push(DirectoryItem {
                    path: entry.path().to_path_buf(),
                    status: Status::Initial,
                });
            }
        }
    }
}

fn main() -> io::Result<()> {
    let configuration = get_configuration();

    let mut app = App::new();

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    app.find_directory_items(configuration.directory);

    app.run(terminal);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

impl App {
    fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        loop {
            self.draw(&mut terminal);

            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Char('q') | Esc => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }
}

const TODO_HEADER_BG: Color = tailwind::BLUE.c950;
const NORMAL_ROW_COLOR: Color = tailwind::SLATE.c950;
const ALT_ROW_COLOR: Color = tailwind::SLATE.c900;
const SELECTED_STYLE_FG: Color = tailwind::BLUE.c300;
const TEXT_COLOR: Color = tailwind::SLATE.c200;
const COMPLETED_TEXT_COLOR: Color = tailwind::GREEN.c500;

impl DirectoryItem {
    fn to_list_item(&self, index: usize) -> ListItem {
        let bg_color = match index % 2 {
            0 => NORMAL_ROW_COLOR,
            _ => ALT_ROW_COLOR,
        };
        let directory_string = self.path.to_str().unwrap();
        let line = match self.status {
            Status::Initial => Line::styled(format!(" ☐ {}", directory_string), TEXT_COLOR),
            Status::InProgress => {
                Line::styled(format!(" ....loading {}", directory_string), TEXT_COLOR)
            }
            Status::Deleted => Line::styled(
                format!(" ✓ {}", directory_string),
                (COMPLETED_TEXT_COLOR, bg_color),
            ),
        };

        ListItem::new(line).bg(bg_color)
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create a space for header, todo list and the footer.
        let vertical = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ]);
        let [header_area, rest_area, footer_area] = vertical.areas(area);

        // We create two blocks, one is for the header (outer) and the other is for list (inner).
        let outer_block = Block::default()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(TODO_HEADER_BG)
            .title("Purrge 🐱")
            .title_alignment(Alignment::Center);
        let inner_block = Block::default()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(NORMAL_ROW_COLOR);

        // We get the inner area from outer_block. We'll use this area later to render the table.
        let outer_area = area;
        let inner_area = outer_block.inner(outer_area);

        // We can render the header in outer_area.
        outer_block.render(outer_area, buf);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .items
            .items
            .iter()
            .enumerate()
            .map(|(i, todo_item)| todo_item.to_list_item(i))
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(inner_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(SELECTED_STYLE_FG),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We can now render the item list
        // (look careful we are using StatefulWidget's render.)
        // ratatui::widgets::StatefulWidget::render as stateful_render
        StatefulWidget::render(items, inner_area, buf, &mut self.items.state);
    }
}
