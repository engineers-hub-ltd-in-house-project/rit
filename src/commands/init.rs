use anyhow::Result;
use colored::*;
use std::fs;
use std::path::Path;

pub fn run() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    
    println!("{}", "Initializing rit repository...".bright_green().bold());
    
    // Check if already initialized
    if current_dir.join(".git").exists() {
        println!("{}", "Repository already initialized with git".yellow());
        return Ok(());
    }
    
    if current_dir.join(".rit").exists() {
        println!("{}", "Repository already initialized with rit".yellow());
        return Ok(());
    }
    
    // Create .rit directory structure
    let rit_dir = current_dir.join(".rit");
    fs::create_dir_all(&rit_dir)?;
    
    // Create basic rit structure
    fs::create_dir_all(rit_dir.join("objects"))?;
    fs::create_dir_all(rit_dir.join("refs").join("heads"))?;
    fs::create_dir_all(rit_dir.join("refs").join("tags"))?;
    
    // Create HEAD file pointing to main branch
    fs::write(rit_dir.join("HEAD"), "ref: refs/heads/main\n")?;
    
    // Create config file
    let config_content = r#"[core]
    repositoryformatversion = 0
    filemode = true
    bare = false
    logallrefupdates = true
[rit]
    graphical = true
    coloroutput = true
"#;
    fs::write(rit_dir.join("config"), config_content)?;
    
    // Create initial .gitignore if it doesn't exist
    let gitignore_path = current_dir.join(".gitignore");
    if !gitignore_path.exists() {
        let gitignore_content = r#"# Rit repository
.rit/

# OS generated files
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# IDE files
.vscode/
.idea/
*.swp
*.swo
*~
"#;
        fs::write(gitignore_path, gitignore_content)?;
        println!("{}", "Created .gitignore".green());
    }
    
    println!("{}", format!("Initialized empty rit repository in {}", 
        rit_dir.display()).green());
    println!();
    println!("{}", "Try running 'rit status' to see the graphical display!".bright_blue());
    
    Ok(())
} 