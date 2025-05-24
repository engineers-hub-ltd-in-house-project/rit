use anyhow::Result;
use colored::*;
use walkdir::WalkDir;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs;
use ignore::WalkBuilder;

pub fn run() -> Result<()> {
    println!("{}", "rit status".bright_green().bold());
    println!();
    
    let current_dir = std::env::current_dir()?;
    
    // Check if we're in a git/rit repository
    if !is_repo(&current_dir)? {
        println!("{}", "fatal: not a rit repository (or any of the parent directories)".red());
        return Ok(());
    }
    
    // Get the current branch
    let branch = get_current_branch(&current_dir)?;
    println!("{} {}", "On branch".blue(), branch.bright_yellow().bold());
    println!();
    
    // Get file status
    let file_status = get_file_status(&current_dir)?;
    
    if file_status.is_empty() {
        println!("{}", "nothing to commit, working tree clean".green());
        return Ok(());
    }
    
    println!("{}", "Changes in working directory:".bright_blue().bold());
    display_tree_structure(&current_dir, &file_status)?;
    
    Ok(())
}

fn is_repo(path: &Path) -> Result<bool> {
    let git_dir = path.join(".git");
    let rit_dir = path.join(".rit");
    Ok(git_dir.exists() || rit_dir.exists())
}

fn get_current_branch(path: &Path) -> Result<String> {
    // First try git
    if let Ok(repo) = git2::Repository::open(path) {
        if let Ok(head) = repo.head() {
            if let Some(name) = head.shorthand() {
                return Ok(name.to_string());
            }
        }
    }
    
    // Fallback to default
    Ok("main".to_string())
}

#[derive(Debug, Clone)]
enum FileStatus {
    New,
    Modified,
    Deleted,
    Renamed,
}

fn get_file_status(path: &Path) -> Result<Vec<(PathBuf, FileStatus)>> {
    let mut status_list = Vec::new();
    
    // Try to get status from git first
    if let Ok(repo) = git2::Repository::open(path) {
        let mut status_opts = git2::StatusOptions::new();
        status_opts.include_untracked(true);
        
        if let Ok(statuses) = repo.statuses(Some(&mut status_opts)) {
            for entry in statuses.iter() {
                if let Some(file_path) = entry.path() {
                    let path_buf = PathBuf::from(file_path);
                    
                    // Check if this file should be ignored by .ritignore
                    if !is_ignored_by_rit(path, &path_buf)? {
                        let status = match entry.status() {
                            s if s.contains(git2::Status::WT_NEW) => FileStatus::New,
                            s if s.contains(git2::Status::WT_MODIFIED) => FileStatus::Modified,
                            s if s.contains(git2::Status::WT_DELETED) => FileStatus::Deleted,
                            s if s.contains(git2::Status::WT_RENAMED) => FileStatus::Renamed,
                            _ => FileStatus::Modified,
                        };
                        status_list.push((path_buf, status));
                    }
                }
            }
        }
    }
    
    // If no git status or empty, show files respecting .gitignore and .ritignore
    if status_list.is_empty() {
        let mut walker = WalkBuilder::new(path);
        walker.max_depth(Some(3));
        walker.hidden(false); // Show hidden files but respect ignore files
        
        // Add custom ignore file for .ritignore
        let ritignore_path = path.join(".ritignore");
        if ritignore_path.exists() {
            walker.add_ignore(ritignore_path);
        }
        
        for result in walker.build() {
            if let Ok(entry) = result {
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    if let Ok(relative_path) = entry.path().strip_prefix(path) {
                        // Skip target directory for Rust projects
                        if !relative_path.to_string_lossy().starts_with("target/") {
                            status_list.push((relative_path.to_path_buf(), FileStatus::New));
                        }
                    }
                }
            }
        }
        
        // Limit to 15 files for display
        status_list.truncate(15);
    }
    
    Ok(status_list)
}

fn is_ignored_by_rit(base_path: &Path, file_path: &PathBuf) -> Result<bool> {
    let ritignore_path = base_path.join(".ritignore");
    
    if ritignore_path.exists() {
        if let Ok(ritignore_content) = std::fs::read_to_string(&ritignore_path) {
            let path_str = file_path.to_string_lossy();
            
            for line in ritignore_content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    // Simple pattern matching
                    if line.starts_with('*') && line.len() > 1 {
                        let pattern = &line[1..]; // Remove the *
                        if path_str.ends_with(pattern) {
                            return Ok(true);
                        }
                    } else if line.ends_with('/') {
                        let dir_pattern = &line[..line.len()-1];
                        if path_str.starts_with(dir_pattern) {
                            return Ok(true);
                        }
                    } else {
                        if path_str.contains(line) || path_str.starts_with(line) {
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }
    
    Ok(false)
}

fn display_tree_structure(base_path: &Path, files: &[(PathBuf, FileStatus)]) -> Result<()> {
    let mut tree_map: std::collections::BTreeMap<String, Vec<(String, FileStatus)>> = std::collections::BTreeMap::new();
    
    // Group files by directory
    for (file_path, status) in files {
        if let Some(parent) = file_path.parent() {
            let dir_key = if parent == Path::new("") {
                ".".to_string()
            } else {
                parent.to_string_lossy().to_string()
            };
            
            let file_name = file_path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
                
            tree_map.entry(dir_key)
                .or_insert_with(Vec::new)
                .push((file_name, status.clone()));
        }
    }
    
    // Display the tree structure
    for (dir, files_in_dir) in tree_map {
        if dir != "." {
            println!("{}", format!("{}/", dir).bright_blue().bold());
        }
        
        for (i, (file_name, status)) in files_in_dir.iter().enumerate() {
            let is_last = i == files_in_dir.len() - 1;
            let connector = if is_last { "└──" } else { "├──" };
            let prefix = if dir != "." { "│   " } else { "" };
            
            let status_symbol = match status {
                FileStatus::New => "??".bright_green(),
                FileStatus::Modified => " M".bright_yellow(),
                FileStatus::Deleted => " D".bright_red(),
                FileStatus::Renamed => " R".bright_cyan(),
            };
            
            let file_color = match status {
                FileStatus::New => file_name.bright_green(),
                FileStatus::Modified => file_name.bright_yellow(),
                FileStatus::Deleted => file_name.bright_red(),
                FileStatus::Renamed => file_name.bright_cyan(),
            };
            
            println!("{}{} {} {}", 
                prefix,
                connector.cyan(),
                status_symbol,
                file_color
            );
        }
        
        if dir != "." {
            println!();
        }
    }
    
    Ok(())
} 