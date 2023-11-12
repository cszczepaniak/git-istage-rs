use git2::StatusOptions;

fn main() {
    println!("Hello, world!");

    let repo = git2::Repository::open(".").unwrap();
    let statuses = repo
        .statuses(Some(
            StatusOptions::default().renames_index_to_workdir(true),
        ))
        .unwrap();

    for status in statuses.iter() {
        println!("{:?}", status.path())
    }
}
