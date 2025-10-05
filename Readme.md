### Rust Compiler

- rustup update
- rustc --version

### How to Run:

- Run cargo run -- add "Buy groceries" to add a task.
- Use commands like cargo run -- list, cargo run -- complete 0, or cargo run -- delete 0.

### Key Features:

- Tasks are stored in tasks.json and persist between runs.
- Supports adding, listing, completing, and deleting tasks.
- Uses clap for intuitive CLI parsing and serde for JSON handling.
