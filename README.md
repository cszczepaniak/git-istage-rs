# git-istage-rs
A port of Immo Landwerth's [git-istage](https://github.com/terrajobst/git-istage/) to Rust.

### Why port this tool?
I really like `git-istage`, and I thought it'd be a fun learning experience to try to port it to Rust.

### Why Rust?
I first tried to port `git-stage` to Go ([here](https://github.com/cszczepaniak/go-istage)). However, I found it incredibly painful to depend on an external C library in Go (libgit2).
This is probably mostly due to my lack of knowledge and experience with this kind of thing. I have to say though, because of Rust's build scripts (and the existence of git2-rs), it 
was completely trivial to add this dependency in Rust.
