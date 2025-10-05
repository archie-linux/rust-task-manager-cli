use clap::{Parser, Subcommand};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use anyhow::{Result};

#[derive(Serialize, Deserialize)]
struct Task {
    id: u32,
    description: String,
    completed: bool,
}

#[derive(Parser)]
#[command(name = "tasker")]
#[command(about = "A simple CLI task manager with SQLite")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { description: String },
    List,
    Complete { id: u32 },
    Delete { id: u32 },
}

const DB_PATH: &str = "tasks.db";

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

fn add_task(conn: &Connection, description: String) -> Result<()> {
    conn.execute(
        "INSERT INTO tasks (description, completed) VALUES (?1, ?2)",
        params![description, false],
    )?;
    println!("Added task with ID: {}", conn.last_insert_rowid());
    Ok(())
}

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

fn delete_task(conn: &Connection, id: u32) -> Result<()> {
    let rows_affected = conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
    if rows_affected == 0 {
        println!("Task {} not found", id);
    } else {
        println!("Deleted task: {}", id);
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = init_db()?;

    match cli.command {
        Commands::Add { description } => add_task(&conn, description)?,
        Commands::List => list_tasks(&conn)?,
        Commands::Complete { id } => complete_task(&conn, id)?,
        Commands::Delete { id } => delete_task(&conn, id)?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_complete_task() -> Result<()> {
        let conn = setup_test_db();
        add_task(&conn, "Test task".to_string())?;
        complete_task(&conn, 1)?;
        let completed: bool = conn.query_row(
            "SELECT completed FROM tasks WHERE id = 1",
            [],
            |row| row.get(0),
        )?;
        assert!(completed);
        Ok(())
    }

    #[test]
    fn test_delete_task() -> Result<()> {
        let conn = setup_test_db();
        add_task(&conn, "Test task".to_string())?;
        delete_task(&conn, 1)?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))?;
        assert_eq!(count, 0);
        Ok(())
    }

    #[test]
    fn test_list_tasks_empty() -> Result<()> {
        let conn = setup_test_db();
        // Redirect stdout to capture output
        // let output = std::io::sink();
        let _ = list_tasks(&conn)?;
        // Since no tasks, no assertion on output; just ensure it runs
        Ok(())
    }
}
