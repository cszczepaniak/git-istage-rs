use std::fs;

use git2::StatusOptions;

use crate::status::StatusEntry;

#[derive(Copy, Clone)]
pub enum FileStatusKind {
    Unstaged,
    Staged,
}

impl From<FileStatusKind> for StatusOptions {
    fn from(value: FileStatusKind) -> Self {
        match value {
            FileStatusKind::Unstaged => {
                let mut opts = StatusOptions::default();
                opts.renames_index_to_workdir(true)
                    .include_untracked(true)
                    .recurse_untracked_dirs(true);
                opts
            }
            FileStatusKind::Staged => {
                let mut opts = StatusOptions::default();
                opts.renames_head_to_index(true);
                opts
            }
        }
    }
}

pub fn get_file_statuses(kind: FileStatusKind) -> anyhow::Result<Vec<StatusEntry>> {
    let repo = git2::Repository::discover(".")?;
    let d = repo.statuses(Some(&mut kind.into()))?;

    let wd = repo.workdir().ok_or(anyhow::anyhow!("what"))?;
    let abs_path = fs::canonicalize(wd)?.to_string_lossy().to_string();

    Ok(d.iter()
        .filter_map(|st| match kind {
            FileStatusKind::Unstaged => st.index_to_workdir(),
            FileStatusKind::Staged => st.head_to_index(),
        })
        .map(|st| (abs_path.clone(), st))
        .map(StatusEntry::from)
        .collect())
}
