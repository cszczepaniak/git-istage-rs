use std::process;

use git2::{Delta, DiffDelta};
use tui::style::Color;

pub struct StatusEntry {
    pub old_file: String,
    pub new_file: String,
    pub status: Status,
}

impl<'a> From<DiffDelta<'a>> for StatusEntry {
    fn from(value: DiffDelta<'a>) -> Self {
        Self {
            old_file: value
                .old_file()
                .path()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            new_file: value
                .new_file()
                .path()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            status: value.status().into(),
        }
    }
}

impl StatusEntry {
    pub fn pretty_string(&self) -> String {
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

    pub fn add_to_git(&self) -> anyhow::Result<()> {
        let mut cmd = process::Command::new("git");
        cmd.arg("add");

        match self.status {
            Status::Renamed => cmd.args([&self.old_file, &self.new_file]),
            _ => cmd.arg(&self.new_file),
        };

        cmd.output()?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub enum Status {
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
