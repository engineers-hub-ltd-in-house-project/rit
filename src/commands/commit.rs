use anyhow::Result;
use colored::*;
use chrono::{DateTime, Local};
use std::process::Command;

pub fn run(message: String) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    
    println!("{}", "Creating commit...".bright_green().bold());
    println!();
    
    // Check if we're in a repository
    if !current_dir.join(".git").exists() && !current_dir.join(".rit").exists() {
        println!("{}", "fatal: not a rit repository".red());
        return Ok(());
    }
    
    // Get commit info
    let timestamp = Local::now();
    let author = get_author_info()?;
    
    // Display commit information in a graphical way
    println!("{}", "┌─ Commit Information".bright_blue().bold());
    println!("{} {}", "│ Message:".bright_blue(), message.bright_white().bold());
    println!("{} {}", "│ Author: ".bright_blue(), author.bright_yellow());
    println!("{} {}", "│ Date:   ".bright_blue(), timestamp.format("%Y-%m-%d %H:%M:%S").to_string().bright_cyan());
    println!("{}", "│".bright_blue());
    
    // Show files being committed (mock implementation)
    println!("{}", "│ Files in this commit:".bright_blue());
    show_commit_tree(&current_dir)?;
    
    println!("{}", "└─ Commit created successfully!".bright_green().bold());
    println!();
    
    // Create a simple commit hash (mock)
    let commit_hash = generate_commit_hash(&message, &timestamp.to_rfc3339())?;
    println!("{} {}", 
        "Commit hash:".bright_blue(), 
        commit_hash.bright_yellow().bold()
    );
    
    println!();
    println!("{}", "Run 'rit log' to see the commit history.".bright_blue());
    
    Ok(())
}

fn get_author_info() -> Result<String> {
    // Try to get git config first
    if let Ok(output) = Command::new("git")
        .args(&["config", "user.name"])
        .output() 
    {
        if output.status.success() {
            if let Ok(name) = String::from_utf8(output.stdout) {
                let name = name.trim();
                if !name.is_empty() {
                    if let Ok(output) = Command::new("git")
                        .args(&["config", "user.email"])
                        .output() 
                    {
                        if output.status.success() {
                            if let Ok(email) = String::from_utf8(output.stdout) {
                                let email = email.trim();
                                if !email.is_empty() {
                                    return Ok(format!("{} <{}>", name, email));
                                }
                            }
                        }
                    }
                    return Ok(name.to_string());
                }
            }
        }
    }
    
    // Fallback to environment variables
    if let Ok(user) = std::env::var("USER") {
        Ok(format!("{} <{}@localhost>", user, user))
    } else {
        Ok("Unknown User <unknown@localhost>".to_string())
    }
}

fn show_commit_tree(base_path: &std::path::Path) -> Result<()> {
    use walkdir::WalkDir;
    
    let mut file_count = 0;
    let files: Vec<_> = WalkDir::new(base_path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !e.path().to_string_lossy().contains(".git"))
        .filter(|e| !e.path().to_string_lossy().contains(".rit"))
        .filter(|e| !e.path().to_string_lossy().contains("target"))
        .take(5)
        .collect();
    
    for (i, entry) in files.iter().enumerate() {
        if let Ok(relative_path) = entry.path().strip_prefix(base_path) {
            let is_last = i == files.len() - 1;
            let connector = if is_last { "└──" } else { "├──" };
            
            println!("{} {} {}", 
                "│".bright_blue(),
                connector.cyan(), 
                relative_path.to_string_lossy().bright_green()
            );
            file_count += 1;
        }
    }
    
    if files.len() == 5 {
        println!("{} {} {}", 
            "│".bright_blue(),
            "└── ...".cyan(), 
            "(and more files)".bright_black()
        );
    }
    
    if file_count == 0 {
        println!("{} {}", 
            "│".bright_blue(),
            "└── No files to commit".yellow()
        );
    }
    
    Ok(())
}

fn generate_commit_hash(message: &str, timestamp: &str) -> Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    message.hash(&mut hasher);
    timestamp.hash(&mut hasher);
    
    let hash = hasher.finish();
    Ok(format!("{:x}", hash)[..8].to_string())
} 