use anyhow::Result;
use colored::*;
use chrono::{DateTime, Local};

pub fn run() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    
    println!("{}", "Commit History".bright_green().bold());
    println!();
    
    // Check if we're in a repository
    if !current_dir.join(".git").exists() && !current_dir.join(".rit").exists() {
        println!("{}", "fatal: not a rit repository".red());
        return Ok(());
    }
    
    // Try to get git log first, if available
    if current_dir.join(".git").exists() {
        display_git_log()?;
    } else {
        display_mock_log()?;
    }
    
    Ok(())
}

fn display_git_log() -> Result<()> {
    use std::process::Command;
    
    if let Ok(output) = Command::new("git")
        .args(&["log", "--oneline", "--graph", "--decorate", "--color=always", "-10"])
        .output() 
    {
        if output.status.success() {
            if let Ok(log_output) = String::from_utf8(output.stdout) {
                if !log_output.trim().is_empty() {
                    println!("{}", "Git commit history:".bright_blue().bold());
                    println!();
                    println!("{}", log_output);
                    return Ok(());
                }
            }
        }
    }
    
    // Fallback to mock log
    display_mock_log()
}

fn display_mock_log() -> Result<()> {
    println!("{}", "Rit commit history (example):".bright_blue().bold());
    println!();
    
    // Mock commit data
    let commits = vec![
        ("a1b2c3d4", "Initial commit with rit structure", "2 hours ago", "main"),
        ("e5f6g7h8", "Add graphical status display", "1 hour ago", ""),
        ("i9j0k1l2", "Implement tree visualization", "30 minutes ago", ""),
        ("m3n4o5p6", "Add colorful output support", "15 minutes ago", "HEAD"),
    ];
    
    for (i, (hash, message, time, branch)) in commits.iter().enumerate() {
        let is_head = branch.contains("HEAD");
        let is_main = branch.contains("main");
        
        // Draw commit graph
        let connector = match i {
            0 => "│",
            _ => "│",
        };
        
        let commit_symbol = if is_head { "●" } else { "○" };
        let commit_color = if is_head { 
            commit_symbol.bright_yellow() 
        } else { 
            commit_symbol.bright_blue() 
        };
        
        // Commit line
        print!("{} {} ", connector.bright_blue(), commit_color);
        print!("{} ", hash.bright_yellow());
        print!("{}", message.bright_white());
        
        if !branch.is_empty() {
            if is_head {
                print!(" {}", format!("({})", branch).bright_red().bold());
            } else if is_main {
                print!(" {}", format!("({})", branch).bright_green().bold());
            }
        }
        
        println!();
        
        // Additional info line
        println!("{} {} {}", 
            "│".bright_blue(),
            "└─".cyan(),
            time.bright_black()
        );
        
        if i < commits.len() - 1 {
            println!("{}", "│".bright_blue());
        }
    }
    
    println!();
    println!("{}", "Legend:".bright_blue().bold());
    println!("{} Current HEAD", "●".bright_yellow());
    println!("{} Previous commits", "○".bright_blue());
    println!();
    println!("{}", "Use 'rit status' to see current working directory state.".bright_blue());
    
    Ok(())
} 