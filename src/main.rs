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

    for status in d.iter().filter_map(|st| st.index_to_workdir()) {
        println!(
            "{:?} -> {:?} [{:?}]",
            status.old_file().path(),
            status.new_file().path(),
            status.status(),
        );
    }

    println!("=== HEAD TO INDEX ===");

    for status in d.iter().filter_map(|st| st.head_to_index()) {
        println!(
            "{:?} -> {:?} [{:?}]",
            status.old_file().path(),
            status.new_file().path(),
            status.status(),
        );
    }
}
