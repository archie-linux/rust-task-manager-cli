use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs::{File};
use std::io::{Read, Write};
use anyhow::Result;

#[derive(Serialize, Deserialize)]
struct Task {
    id: u32,
    description: String,
    completed: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct TaskList {
    tasks: Vec<Task>,
    next_id: u32,
}

#[derive(Parser)]
#[command(name = "tasker")]
#[command(about = "A simple CLI task manager")]
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

const FILE_PATH: &str = "tasks.json";

fn load_tasks() -> Result<TaskList> {
    let mut file = File::open(FILE_PATH).or_else(|_| File::create(FILE_PATH))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    if contents.is_empty() {
        Ok(TaskList::default())
    } else {
        Ok(serde_json::from_str(&contents)?)
    }
}

fn save_tasks(tasks: &TaskList) -> Result<()> {
    let json = serde_json::to_string_pretty(tasks)?;
    let mut file = File::create(FILE_PATH)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut task_list = load_tasks()?;

    match cli.command {
        Commands::Add { description } => {
            let task = Task {
                id: task_list.next_id,
                description,
                completed: false,
            };
            task_list.tasks.push(task);
            task_list.next_id += 1;
            println!("Added task: {}", task_list.next_id - 1);
        }
        Commands::List => {
            for task in &task_list.tasks {
                let status = if task.completed { "[x]" } else { "[ ]" };
                println!("{} {}: {}", task.id, status, task.description);
            }
        }
        Commands::Complete { id } => {
            if let Some(task) = task_list.tasks.iter_mut().find(|t| t.id == id) {
                task.completed = true;
                println!("Completed task: {}", id);
            } else {
                println!("Task {} not found", id);
            }
        }
        Commands::Delete { id } => {
            task_list.tasks.retain(|t| t.id != id);
            println!("Deleted task: {}", id);
        }
    }

    save_tasks(&task_list)?;
    Ok(())
}
