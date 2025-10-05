# CLI Task Manager - Code Documentation

## Module Structure

```rust
src/
└── main.rs  // Complete application implementation
```

## Dependencies and Imports

```rust
use clap::{Parser, Subcommand};        // CLI argument parsing
use rusqlite::{params, Connection};    // SQLite database operations
use serde::{Deserialize, Serialize};   // JSON serialization support
use anyhow::{Result};                  // Error handling
```

## Data Structures

### Task Struct
```rust
#[derive(Serialize, Deserialize)]
struct Task {
    id: u32,           // Primary key, auto-incremented by SQLite
    description: String, // Task description text
    completed: bool,    // Task completion status
}
```

**Purpose**: Represents a single task in the system
- **Serialization**: Supports JSON export/import via serde
- **Database Mapping**: Maps directly to SQLite table schema
- **Type Safety**: Uses u32 for ID to match SQLite INTEGER PRIMARY KEY

### CLI Interface Structures

#### Main CLI Parser
```rust
#[derive(Parser)]
#[command(name = "tasker")]
#[command(about = "A simple CLI task manager with SQLite")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```

**Attributes Explained**:
- `#[derive(Parser)]`: Generates command-line parsing code
- `#[command(name = "tasker")]`: Sets binary name in help text
- `#[command(about = "...")]`: Description shown in help

#### Command Enumeration
```rust
#[derive(Subcommand)]
enum Commands {
    Add { description: String },    // Add new task with description
    List,                          // List all tasks (no parameters)
    Complete { id: u32 },          // Mark task as complete by ID
    Delete { id: u32 },            // Delete task by ID
}
```

**Design Notes**:
- Each variant represents a CLI subcommand
- Parameters are embedded as struct fields
- `String` for flexible description input
- `u32` for numeric task IDs with validation

## Constants

```rust
const DB_PATH: &str = "tasks.db";
```

**Purpose**: Centralized database file location
- **Hardcoded Path**: Creates file in current working directory
- **Future Enhancement**: Could be made configurable via CLI argument

## Core Functions

### Database Initialization

```rust
fn init_db() -> Result<Connection> {
    let conn = Connection::open(DB_PATH)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            completed BOOLEAN NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}
```

**Function Analysis**:
- **Return Type**: `Result<Connection>` for error propagation
- **Database Creation**: Uses `IF NOT EXISTS` for idempotent operation
- **Schema Design**:
  - `id INTEGER PRIMARY KEY`: Auto-incrementing unique identifier
  - `description TEXT NOT NULL`: Required task description
  - `completed BOOLEAN NOT NULL`: Required completion status
- **Error Handling**: Uses `?` operator for automatic error propagation

### Task Management Functions

#### Add Task Function
```rust
fn add_task(conn: &Connection, description: String) -> Result<()> {
    conn.execute(
        "INSERT INTO tasks (description, completed) VALUES (?1, ?2)",
        params![description, false],
    )?;
    println!("Added task with ID: {}", conn.last_insert_rowid());
    Ok(())
}
```

**Implementation Details**:
- **Parameters**: 
  - `conn: &Connection`: Borrowed database connection
  - `description: String`: Owned string to avoid lifetime issues
- **SQL**: Uses parameterized query to prevent SQL injection
- **Default State**: All new tasks start as incomplete (`false`)
- **User Feedback**: Prints the auto-generated task ID
- **Return**: `Result<()>` for error handling

#### List Tasks Function
```rust
fn list_tasks(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT id, description, completed FROM tasks")?;
    let tasks = stmt.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            description: row.get(1)?,
            completed: row.get(2)?,
        })
    })?;

    for task in tasks {
        let task = task?;
        let status = if task.completed { "[x]" } else { "[ ]" };
        println!("{} {}: {}", task.id, status, task.description);
    }
    Ok(())
}
```

**Function Breakdown**:
1. **Statement Preparation**: Compiles SQL query for execution
2. **Query Mapping**: Transforms rows into `Task` structs
3. **Row Processing**: 
   - `row.get(0)?`: Gets ID column with error handling
   - `row.get(1)?`: Gets description column
   - `row.get(2)?`: Gets completed column
4. **Display Logic**: 
   - `[x]` for completed tasks
   - `[ ]` for incomplete tasks
5. **Error Propagation**: Each `?` can short-circuit on database errors

#### Complete Task Function
```rust
fn complete_task(conn: &Connection, id: u32) -> Result<()> {
    let rows_affected = conn.execute(
        "UPDATE tasks SET completed = ?1 WHERE id = ?2",
        params![true, id],
    )?;
    if rows_affected == 0 {
        println!("Task {} not found", id);
    } else {
        println!("Completed task: {}", id);
    }
    Ok(())
}
```

**Logic Flow**:
1. **Update Execution**: Sets `completed = true` for specified ID
2. **Affected Rows Check**: SQLite returns number of modified rows
3. **Validation**: 
   - `rows_affected == 0`: No task with that ID exists
   - `rows_affected > 0`: Task successfully updated
4. **User Feedback**: Different messages for success/failure cases

#### Delete Task Function
```rust
fn delete_task(conn: &Connection, id: u32) -> Result<()> {
    let rows_affected = conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
    if rows_affected == 0 {
        println!("Task {} not found", id);
    } else {
        println!("Deleted task: {}", id);
    }
    Ok(())
}
```

**Implementation Notes**:
- **Similar Pattern**: Follows same validation logic as `complete_task`
- **Permanent Deletion**: No soft delete or recovery mechanism
- **Safety**: Uses parameterized query to prevent SQL injection

## Main Function

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();           // Parse command line arguments
    let conn = init_db()?;            // Initialize database connection
    
    match cli.command {               // Match on parsed command
        Commands::Add { description } => add_task(&conn, description)?,
        Commands::List => list_tasks(&conn)?,
        Commands::Complete { id } => complete_task(&conn, id)?,
        Commands::Delete { id } => delete_task(&conn, id)?,
    }
    
    Ok(())
}
```

**Execution Flow**:
1. **CLI Parsing**: `Cli::parse()` processes `std::env::args()`
2. **Database Setup**: `init_db()` ensures database and table exist
3. **Command Dispatch**: Pattern matching routes to appropriate function
4. **Error Propagation**: Any function error bubbles up to main
5. **Automatic Cleanup**: Connection closed when `conn` goes out of scope

## Test Module

### Test Helper Function
```rust
fn setup_test_db() -> Connection {
    let conn = Connection::open(":memory:").unwrap();
    conn.execute(
        "CREATE TABLE tasks (
            id INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            completed BOOLEAN NOT NULL
        )",
        [],
    ).unwrap();
    conn
}
```

**Test Infrastructure**:
- **In-Memory Database**: `:memory:` creates temporary SQLite database
- **Isolation**: Each test gets fresh database state
- **Schema Consistency**: Matches production table structure
- **Error Handling**: Uses `.unwrap()` since test panics are acceptable

### Individual Test Functions

#### Test Add Task
```rust
#[test]
fn test_add_task() -> Result<()> {
    let conn = setup_test_db();
    add_task(&conn, "Test task".to_string())?;
    let mut stmt = conn.prepare("SELECT description, completed FROM tasks WHERE id = 1")?;
    let task = stmt.query_row([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?))
    })?;
    assert_eq!(task, ("Test task".to_string(), false));
    Ok(())
}
```

**Test Strategy**:
1. **Setup**: Create isolated test database
2. **Action**: Call function under test
3. **Verification**: Query database to verify state
4. **Assertion**: Compare expected vs actual values

**Type Annotations**: 
- `row.get::<_, String>(0)?`: Explicitly specify return type
- `row.get::<_, bool>(1)?`: Type inference with explicit bool

#### Test Complete Task
```rust
#[test]
fn test_complete_task() -> Result<()> {
    let conn = setup_test_db();
    add_task(&conn, "Test task".to_string())?;  // Setup: Create task
    complete_task(&conn, 1)?;                   // Action: Complete it
    let completed: bool = conn.query_row(       // Verify: Check status
        "SELECT completed FROM tasks WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    assert!(completed);                         // Assert: Must be true
    Ok(())
}
```

**Test Flow**:
1. **Setup State**: Create task to operate on
2. **Execute Operation**: Call the function being tested
3. **Query Result**: Fetch specific field to verify
4. **Assert Outcome**: Use `assert!()` for boolean verification

#### Test Delete Task
```rust
#[test]
fn test_delete_task() -> Result<()> {
    let conn = setup_test_db();
    add_task(&conn, "Test task".to_string())?;
    delete_task(&conn, 1)?;
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))?;
    assert_eq!(count, 0);
    Ok(())
}
```

**Verification Strategy**:
- **Count Query**: `SELECT COUNT(*)` returns total rows
- **Expected State**: After deletion, should be 0 tasks
- **Type Handling**: SQLite COUNT returns `i64`

## Error Handling Patterns

### Error Propagation Chain
```rust
main() -> Result<()>
  ├── init_db() -> Result<Connection>
  ├── add_task() -> Result<()>
  ├── list_tasks() -> Result<()>
  ├── complete_task() -> Result<()>
  └── delete_task() -> Result<()>
```

### Error Types Handled
1. **Database Connection Errors**: File permissions, disk space
2. **SQL Execution Errors**: Syntax errors, constraint violations
3. **Data Type Errors**: Column type mismatches
4. **I/O Errors**: File system issues

### Error Recovery
- **None implemented**: Errors bubble up and terminate program
- **Future Enhancement**: Could add retry logic for transient errors

## Memory Management

### String Handling
- **Owned Strings**: Functions take `String` (owned) for descriptions
- **Borrowed References**: Database connections passed as `&Connection`
- **String Literals**: SQL queries use `&str` constants

### Connection Lifecycle
- **Single Connection**: One connection per program execution
- **Automatic Cleanup**: Connection closed when dropped
- **Transaction Behavior**: Each operation is auto-committed

## Performance Considerations

### Database Operations
- **No Connection Pooling**: Single-threaded, single connection
- **No Prepared Statement Caching**: Statements prepared per call
- **No Transactions**: Each operation commits immediately

### Memory Usage
- **Minimal Overhead**: Structures are simple with basic types
- **No Caching**: Tasks not cached in memory
- **Stream Processing**: List operation processes rows sequentially

## Security Considerations

### SQL Injection Prevention
```rust
// Good: Parameterized query
conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])

// Bad: String interpolation (not used)
// conn.execute(&format!("DELETE FROM tasks WHERE id = {}", id))
```

### Input Validation
- **Type Safety**: Rust type system prevents many errors
- **No Length Limits**: Task descriptions can be arbitrarily long
- **No Content Filtering**: Accepts any UTF-8 text

## Potential Improvements

### Code Quality
1. **Error Messages**: More descriptive error reporting
2. **Input Validation**: Length limits, content restrictions
3. **Configuration**: Configurable database path
4. **Logging**: Structured logging instead of println!

### Performance
1. **Connection Pooling**: Reuse database connections
2. **Prepared Statements**: Cache compiled queries
3. **Transactions**: Batch operations for consistency

### Features
1. **Task Editing**: Modify task descriptions
2. **Priorities**: Add task priority levels
3. **Due Dates**: Time-based task management
4. **Categories**: Organize tasks by category