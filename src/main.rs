use git2::{DiffOptions, StatusOptions};

fn main() {
    println!("Hello, world!");

    let repo = git2::Repository::open(".").unwrap();
    let d = repo
        .statuses(Some(
            StatusOptions::default()
                .renames_index_to_workdir(true)
                .include_untracked(true)
                .recurse_untracked_dirs(true),
        ))
        .unwrap();

    println!("=== INDEX TO WORKDIR ===");

    for status in d.iter().filter(|st| st.index_to_workdir().is_some()) {
        println!(
            "{:?} -> {:?} [{:?}]",
            status.index_to_workdir().unwrap().old_file().path(),
            status.index_to_workdir().unwrap().new_file().path(),
            status.status(),
        );
    }

    println!("=== HEAD TO INDEX ===");

    for status in d.iter().filter(|st| st.head_to_index().is_some()) {
        println!(
            "{:?} -> {:?} [{:?}]",
            status.head_to_index().unwrap().old_file().path(),
            status.head_to_index().unwrap().new_file().path(),
            status.status(),
        );
    }
}
