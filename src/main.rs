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

fn main() -> anyhow::Result<()> {
    println!("Hello, world!");

    let repo = git2::Repository::open(".")?;
    let d = repo.statuses(Some(
        StatusOptions::default()
            .renames_index_to_workdir(true)
            .include_untracked(true)
            .recurse_untracked_dirs(true),
    ))?;

    // println!("=== INDEX TO WORKDIR ===");

    // for status in d.iter().filter_map(|st| st.index_to_workdir()) {
    //     println!(
    //         "{:?} -> {:?} [{:?}]",
    //         status.old_file().path(),
    //         status.new_file().path(),
    //         status.status(),
    //     );
    // }

    // println!("=== HEAD TO INDEX ===");

    // for status in d.iter().filter_map(|st| st.head_to_index()) {
    //     println!(
    //         "{:?} -> {:?} [{:?}]",
    //         status.old_file().path(),
    //         status.new_file().path(),
    //         status.status(),
    //     );
    // }

    let items = d
        .iter()
        .filter_map(|st| st.index_to_workdir())
        .map(|st| {
            format!(
                "{:?} -> {:?} [{:?}]",
                st.old_file().path(),
                st.new_file().path(),
                st.status()
            )
        })
        .collect::<Vec<String>>();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let app = App::new(items);
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
        StatefulList {
            state: ListState::default(),
            items,
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
    items: StatefulList<String>,
}

impl App {
    fn new(items: Vec<String>) -> App {
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
        .map(|s| ListItem::new(Span::from(s.clone())).style(Style::default().fg(Color::Gray)))
        .collect();

    let list = List::new(items).highlight_style(
        Style::default()
            .bg(Color::Gray)
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    f.render_stateful_widget(list, size, &mut app.items.state);
}
