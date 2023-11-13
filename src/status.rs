use std::{
    fs,
    path::{self, PathBuf},
    process,
};

use git2::{Delta, DiffDelta};
use tui::style::Color;

pub struct StatusEntry {
    repo_root: String,
    pub old_file: String,
    pub new_file: String,
    pub status: Status,
}

impl<'a> From<(String, DiffDelta<'a>)> for StatusEntry {
    fn from(value: (String, DiffDelta<'a>)) -> Self {
        Self {
            repo_root: value.0,
            old_file: value
                .1
                .old_file()
                .path()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            new_file: value
                .1
                .new_file()
                .path()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            status: value.1.status().into(),
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

    fn abs_path_old(&self) -> PathBuf {
        path::Path::new(&self.repo_root).join(&self.old_file)
    }

    fn abs_path_new(&self) -> PathBuf {
        path::Path::new(&self.repo_root).join(&self.new_file)
    }

    pub fn stage_to_index(&self) -> anyhow::Result<()> {
        let mut cmd = process::Command::new("git");
        cmd.arg("add");

        // Assumption: this StatusEntry was obtained by compaing the index to the working directory.
        match self.status {
            Status::Renamed => cmd.args([self.abs_path_old(), self.abs_path_new()]),
            _ => cmd.arg(&self.abs_path_new()),
        };

        cmd.output()?;
        Ok(())
    }

    pub fn reset_from_workdir(&self) -> anyhow::Result<()> {
        // Assumption: this StatusEntry was obtained by compaing the index to the working directory.
        match self.status {
            Status::Untracked => {
                fs::remove_file(self.abs_path_new())?;
            }
            Status::Renamed => {
                fs::remove_file(self.abs_path_new())?;
                process::Command::new("git")
                    .arg("checkout")
                    .arg(self.abs_path_old())
                    .output()?;
            }
            _ => {
                process::Command::new("git")
                    .arg("checkout")
                    .arg(self.abs_path_new())
                    .output()?;
            }
        };

        Ok(())
    }

    pub fn unstage_to_workdir(&self) -> anyhow::Result<()> {
        let mut cmd = process::Command::new("git");

        // Assumption: this StatusEntry was obtained by comparing HEAD to the index.
        match self.status {
            Status::Deleted => {
                cmd.arg("restore").arg("--staged").arg(self.abs_path_new());
            }
            _ => {
                cmd.arg("reset").arg(self.abs_path_new());
            }
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
