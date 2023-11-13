mod git;
mod status;

use std::time::Instant;
use std::{io, time::Duration};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{List, ListItem, ListState},
    Frame, Terminal,
};

use git::get_file_statuses;
use status::StatusEntry;

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let app = App::new(
        get_file_statuses(git::FileStatusKind::Unstaged)?,
        get_file_statuses(git::FileStatusKind::Staged)?,
    );
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

enum AppViewState {
    UnstagedFiles,
    StagedFiles,
}

struct App {
    view_state: AppViewState,
    unstaged_files: StatefulList<StatusEntry>,
    staged_files: StatefulList<StatusEntry>,
}

impl App {
    fn new(unstaged_files: Vec<StatusEntry>, staged_files: Vec<StatusEntry>) -> App {
        App {
            view_state: AppViewState::UnstagedFiles,
            unstaged_files: StatefulList::with_items(unstaged_files),
            staged_files: StatefulList::with_items(staged_files),
        }
    }

    fn curr_file_list(&mut self) -> &mut StatefulList<StatusEntry> {
        match self.view_state {
            AppViewState::UnstagedFiles => &mut self.unstaged_files,
            AppViewState::StagedFiles => &mut self.staged_files,
        }
    }

    fn change_view_state<F>(&mut self, next: AppViewState, mut on_enter: F) -> anyhow::Result<()>
    where
        F: FnMut(&mut App) -> anyhow::Result<()>,
    {
        on_enter(self)?;
        self.view_state = next;
        Ok(())
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
                        if let AppViewState::StagedFiles = app.view_state {
                            continue;
                        }
                        if let Some(item) = app.unstaged_files.current() {
                            item.stage_to_index()?;
                            app.unstaged_files
                                .set_items(get_file_statuses(git::FileStatusKind::Unstaged)?);
                        }
                    }
                    KeyCode::Char('r') => {
                        if let AppViewState::StagedFiles = app.view_state {
                            continue;
                        }
                        if let Some(item) = app.unstaged_files.current() {
                            item.reset_from_workdir()?;
                            app.unstaged_files
                                .set_items(get_file_statuses(git::FileStatusKind::Unstaged)?);
                        }
                    }
                    KeyCode::Char('u') => {
                        if let AppViewState::UnstagedFiles = app.view_state {
                            continue;
                        }
                        if let Some(item) = app.staged_files.current() {
                            item.unstage_to_workdir()?;
                            app.staged_files
                                .set_items(get_file_statuses(git::FileStatusKind::Staged)?);
                        }
                    }
                    KeyCode::Char('t') => match app.view_state {
                        AppViewState::UnstagedFiles => {
                            app.change_view_state(AppViewState::StagedFiles, |app| {
                                app.staged_files
                                    .set_items(get_file_statuses(git::FileStatusKind::Staged)?);
                                Ok(())
                            })?
                        }
                        AppViewState::StagedFiles => {
                            app.change_view_state(AppViewState::UnstagedFiles, |app| {
                                app.unstaged_files
                                    .set_items(get_file_statuses(git::FileStatusKind::Unstaged)?);
                                Ok(())
                            })?
                        }
                    },
                    KeyCode::Down => app.curr_file_list().next(),
                    KeyCode::Up => app.curr_file_list().previous(),
                    KeyCode::Left => app.curr_file_list().unselect(),
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
    match app.view_state {
        AppViewState::UnstagedFiles => files_view(f, &mut app.unstaged_files),
        AppViewState::StagedFiles => files_view(f, &mut app.staged_files),
    }
}

fn files_view<B: Backend>(f: &mut Frame<B>, input: &mut StatefulList<StatusEntry>) {
    let size = f.size();
    let items: Vec<ListItem> = input
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

    f.render_stateful_widget(list, size, &mut input.state);
}
