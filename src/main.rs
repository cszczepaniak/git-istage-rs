mod status;

use std::time::Instant;
use std::{io, time::Duration};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::StatusOptions;
use tui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{List, ListItem, ListState},
    Frame, Terminal,
};

use status::StatusEntry;

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
        .map(StatusEntry::from)
        .collect())
}
