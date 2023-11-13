use std::ffi::OsStr;
use std::process;
use std::time::Instant;
use std::{io, time::Duration};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::{Delta, StatusOptions};
use tui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{List, ListItem, ListState},
    Frame, Terminal,
};

#[derive(Clone, Copy)]
enum Status {
    Unmodified,
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
    Ignored,
    Untracked,
    Conflicted,
    Typechange,
    Unreadable,
}

impl From<Delta> for Status {
    fn from(value: Delta) -> Self {
        match value {
            Delta::Unmodified => Status::Unmodified,
            Delta::Added => Status::Added,
            Delta::Deleted => Status::Deleted,
            Delta::Modified => Status::Modified,
            Delta::Renamed => Status::Renamed,
            Delta::Copied => Status::Copied,
            Delta::Ignored => Status::Ignored,
            Delta::Untracked => Status::Untracked,
            Delta::Typechange => Status::Typechange,
            Delta::Unreadable => Status::Unreadable,
            Delta::Conflicted => Status::Conflicted,
        }
    }
}

impl From<Status> for char {
    fn from(value: Status) -> Self {
        match value {
            Status::Unmodified => ' ',
            Status::Added => 'A',
            Status::Deleted => 'D',
            Status::Modified => 'M',
            Status::Renamed => 'R',
            Status::Copied => 'C',
            Status::Ignored => '!',
            Status::Untracked => 'U',
            Status::Conflicted => 'X',
            Status::Typechange => todo!(),
            Status::Unreadable => todo!(),
        }
    }
}

impl From<Status> for Color {
    fn from(value: Status) -> Self {
        match value {
            Status::Unmodified => Color::White,
            Status::Added => Color::LightGreen,
            Status::Deleted => Color::Red,
            Status::Modified => Color::Yellow,
            Status::Renamed => Color::Cyan,
            Status::Copied => Color::LightBlue,
            Status::Ignored => Color::Gray,
            Status::Untracked => Color::Green,
            Status::Conflicted => Color::LightRed,
            Status::Typechange => todo!(),
            Status::Unreadable => todo!(),
        }
    }
}

struct StatusEntry {
    old_file: String,
    new_file: String,
    status: Status,
}

impl StatusEntry {
    fn pretty_string(&self) -> String {
        match self.status {
            Status::Renamed => format!(
                "{} {} -> {}",
                char::from(self.status),
                self.old_file,
                self.new_file
            ),
            _ => format!("{} {}", char::from(self.status), self.new_file),
        }
    }

    fn add_to_git(&self) -> anyhow::Result<()> {
        match self.status {
            Status::Renamed => add_to_git([&self.old_file, &self.new_file]),
            _ => add_to_git([&self.new_file]),
        }
    }
}

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let app = App::new(get_file_statuses()?);
    let res = run_app(&mut terminal, app, tick_rate);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res?;

    Ok(())
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();
        state.select(Some(0));
        StatefulList { state, items }
    }

    fn set_items(&mut self, items: Vec<T>) {
        self.items = items;

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    self.items.len() - 1
                } else {
                    i
                }
            }
            None => 0,
        };
        self.state.select(Some(i))
    }

    fn current(&self) -> Option<&T> {
        match self.state.selected() {
            Some(i) => Some(&self.items[i]),
            None => None,
        }
    }

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

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct App {
    items: StatefulList<StatusEntry>,
}

impl App {
    fn new(items: Vec<StatusEntry>) -> App {
        App {
            items: StatefulList::with_items(items),
        }
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> anyhow::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('s') => {
                        if let Some(item) = app.items.current() {
                            item.add_to_git()?;
                            app.items.set_items(get_file_statuses()?);
                        }
                    }
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    KeyCode::Left => app.items.unselect(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|s| {
            ListItem::new(Span::styled(
                s.pretty_string(),
                Style::default().fg(s.status.into()),
            ))
            .style(Style::default().fg(Color::Gray))
        })
        .collect();

    let list = List::new(items).highlight_style(
        Style::default()
            .bg(Color::Rgb(75, 75, 75))
            .add_modifier(Modifier::BOLD),
    );

    f.render_stateful_widget(list, size, &mut app.items.state);
}

fn get_file_statuses() -> anyhow::Result<Vec<StatusEntry>> {
    let repo = git2::Repository::open(".")?;
    let d = repo.statuses(Some(
        StatusOptions::default()
            .renames_index_to_workdir(true)
            .include_untracked(true)
            .recurse_untracked_dirs(true),
    ))?;

    Ok(d.iter()
        .filter_map(|st| st.index_to_workdir())
        .map(|st| StatusEntry {
            old_file: st
                .old_file()
                .path()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            new_file: st
                .new_file()
                .path()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            status: st.status().into(),
        })
        .collect())
}

fn add_to_git<I, S>(paths: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    process::Command::new("git")
        .arg("add")
        .args(paths)
        .output()?;
    Ok(())
}
