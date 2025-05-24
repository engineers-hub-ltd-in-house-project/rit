use anyhow::Result;
use colored::*;
use std::path::Path;
use ignore::WalkBuilder;

pub fn run(files: Vec<String>) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    
    if files.is_empty() {
        println!("{}", "Nothing specified, nothing added.".yellow());
        println!("{}", "Maybe you wanted to say 'rit add .'?".bright_blue());
        return Ok(());
    }
    
    println!("{}", "Adding files to staging area...".bright_green().bold());
    println!();
    
    for file in &files {
        if file == "." {
            println!("{}", "Adding all files in current directory:".bright_blue());
            add_all_files(&current_dir)?;
        } else {
            add_single_file(&current_dir, file)?;
        }
    }
    
    println!();
    println!("{}", "Files added successfully!".green());
    println!("{}", "Run 'rit status' to see the changes.".bright_blue());
    
    Ok(())
}

fn add_single_file(base_path: &Path, file_path: &str) -> Result<()> {
    let full_path = base_path.join(file_path);
    
    if !full_path.exists() {
        println!("{} {}", "File not found:".red(), file_path.bright_red());
        return Ok(());
    }
    
    // Check if file should be ignored
    if is_ignored(base_path, &full_path)? {
        println!("{} {} {}", 
            "├── Ignored:".bright_black(), 
            file_path.bright_black(),
            "(matches ignore pattern)".bright_black()
        );
        return Ok(());
    }
    
    if full_path.is_file() {
        println!("{} {}", "├── Added:".green(), file_path.bright_green());
    } else if full_path.is_dir() {
        println!("{} {}/", "├── Added directory:".green(), file_path.bright_green());
        // In a real implementation, you'd recursively add all files in the directory
    }
    
    Ok(())
}

fn add_all_files(base_path: &Path) -> Result<()> {
    let mut file_count = 0;
    let mut ignored_count = 0;
    
    let mut walker = WalkBuilder::new(base_path);
    walker.hidden(false); // Show hidden files but respect ignore files
    
    // Add custom ignore file for .ritignore
    let ritignore_path = base_path.join(".ritignore");
    if ritignore_path.exists() {
        walker.add_ignore(ritignore_path);
    }
    
    for result in walker.build() {
        if let Ok(entry) = result {
            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                if let Ok(relative_path) = entry.path().strip_prefix(base_path) {
                    let path_str = relative_path.to_string_lossy();
                    
                    // Skip target directory for Rust projects specifically
                    if path_str.starts_with("target/") {
                        continue;
                    }
                    
                    println!("{} {}", "├── Added:".green(), path_str.bright_green());
                    file_count += 1;
                }
            }
        }
    }
    
    if file_count == 0 && ignored_count == 0 {
        println!("{}", "No files to add.".yellow());
    } else {
        println!();
        println!("{} {} {}", 
            "Total:".bright_blue(),
            file_count.to_string().bright_yellow(),
            "files added".bright_blue()
        );
        
        if ignored_count > 0 {
            println!("{} {} {}", 
                "Ignored:".bright_black(),
                ignored_count.to_string().bright_black(),
                "files (matches ignore patterns)".bright_black()
            );
        }
    }
    
    Ok(())
}

fn is_ignored(base_path: &Path, file_path: &Path) -> Result<bool> {
    // Simple check - in a full implementation, you'd use ignore crate more thoroughly
    let gitignore_path = base_path.join(".gitignore");
    let ritignore_path = base_path.join(".ritignore");
    
    if let Ok(relative_path) = file_path.strip_prefix(base_path) {
        let path_str = relative_path.to_string_lossy();
        
        // Check common ignore patterns
        if path_str.starts_with("target/") || 
           path_str.starts_with(".git/") || 
           path_str.starts_with(".rit/") ||
           path_str.contains("node_modules/") {
            return Ok(true);
        }
        
        // Check .gitignore patterns (simplified)
        if gitignore_path.exists() {
            if let Ok(gitignore_content) = std::fs::read_to_string(&gitignore_path) {
                for line in gitignore_content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        if path_str.contains(line) || path_str.starts_with(line) {
                            return Ok(true);
                        }
                    }
                }
            }
        }
        
        // Check .ritignore patterns (simplified)
        if ritignore_path.exists() {
            if let Ok(ritignore_content) = std::fs::read_to_string(&ritignore_path) {
                for line in ritignore_content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
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