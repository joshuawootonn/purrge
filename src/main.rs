use anyhow::Result;
use configuration::get_configuration;
use globset::{Glob, GlobSetBuilder};
use walkdir::WalkDir;

use std::{
    fs::remove_dir_all,
    io::{self, stdout},
    path::PathBuf,
};
use tokio::sync::mpsc::{self, UnboundedSender};

use crossterm::{
    event::{self, Event, KeyCode},
    ExecutableCommand,
};
use ratatui::{prelude::*, style::palette::tailwind, terminal, widgets::*};

mod configuration;

pub fn initialize_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        shutdown().unwrap();
        original_hook(panic_info);
    }));
}

fn startup() -> Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    stdout().execute(crossterm::terminal::EnterAlternateScreen)?;
    Ok(())
}
fn shutdown() -> Result<()> {
    stdout().execute(crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let configuration = get_configuration();

    let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(action_tx);

    initialize_panic_handler();
    startup()?;

    app.setup_event_handlers(app.action_tx.clone());

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    app.find_directory_items(configuration.directory);

    loop {
        app.draw(&mut terminal)?;

        if let Some(action) = action_rx.recv().await {
            app.reducer(action);
        }

        if app.should_quit {
            break;
        }
    }

    shutdown()
}

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
    should_quit: bool,
    items: StatefulList,
    action_tx: UnboundedSender<Action>,
}

struct AddDirectoryAction {
    path: PathBuf,
}

struct DeleteDirectoryAction;

struct DeleteDirectoryActionCompleted {
    path: PathBuf,
}

struct SelectNextAction;
struct SelectPrevAction;
struct QuitAction;

enum Action {
    AddDirectory(AddDirectoryAction),
    DeteteDirectory(DeleteDirectoryAction),
    DeleteDirectoryCompleted(DeleteDirectoryActionCompleted),
    SelectNext(SelectNextAction),
    SelectPrev(SelectPrevAction),
    Quit(QuitAction),
    None,
}

impl App {
    fn new(action_tx: UnboundedSender<Action>) -> App {
        App {
            items: StatefulList {
                state: ListState::default(),
                items: vec![],
                last_selected: None,
            },
            action_tx,
            should_quit: false,
        }
    }

    fn find_directory_items(&mut self, path: PathBuf) {
        let mut builder = GlobSetBuilder::new();

        builder.add(Glob::new("**/node_modules").unwrap());
        builder.add(Glob::new("**/dist").unwrap());
        let match_these_glob = builder.build().unwrap();

        let mut builder2 = GlobSetBuilder::new();

        builder2.add(Glob::new("**/node_modules/*").unwrap());
        builder2.add(Glob::new("**/dist/*").unwrap());
        let dont_match_glob = builder2.build().unwrap();

        // TODO: don't keep walking when in excluded directory or hidden directory
        let walker = WalkDir::new(path.to_str().unwrap()).into_iter();
        for entry in walker {
            let entry = entry.unwrap();
            let a = match_these_glob.matches(entry.path()).len();
            let b = dont_match_glob.matches(entry.path()).len();

            if a > 0 && b == 0 {
                self.items.items.push(DirectoryItem {
                    path: entry.path().to_path_buf(),
                    status: Status::Initial,
                });
            }
        }
    }

    fn setup_event_handlers(&mut self, tx: UnboundedSender<Action>) -> tokio::task::JoinHandle<()> {
        let tick_rate = std::time::Duration::from_millis(32);
        tokio::spawn(async move {
            loop {
                let action = if crossterm::event::poll(tick_rate).unwrap() {
                    if let Ok(Event::Key(key)) = crossterm::event::read() {
                        if key.kind == event::KeyEventKind::Press {
                            use KeyCode::*;
                            match key.code {
                                Char('q') | Esc => Action::Quit(QuitAction),
                                Char('j') | Down => Action::SelectNext(SelectNextAction),
                                Char('k') | Up => Action::SelectPrev(SelectPrevAction),
                                Space => Action::DeteteDirectory(DeleteDirectoryAction),
                                _ => Action::None,
                            }
                        } else {
                            Action::None
                        }
                    } else {
                        Action::None
                    }
                } else {
                    Action::None
                };
                if tx.send(action).is_err() {
                    break;
                }
            }
        })
    }

    fn reducer(&mut self, action: Action) {
        match action {
            Action::SelectNext(_) => self.items.next(),
            Action::SelectPrev(_) => self.items.previous(),
            Action::DeteteDirectory(_) => self.items.delete(),
            Action::Quit(_) => self.should_quit = true,
            _ => {}
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }
}

impl StatefulList {
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn delete(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                self.items[i].status = Status::InProgress;
                remove_dir_all(self.items[i].path.to_str().unwrap()).unwrap();
                self.items[i].status = Status::Deleted;

                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
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
