use clap::Parser;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;
use std::io;
use thiserror::Error;
use std::ffi::OsStr;
use indicatif::{ProgressBar, ProgressStyle};
use colored::Colorize;
use dirs::download_dir; // For detecting Downloads folder

/// A simple CLI tool to organize files by type
#[derive(Parser)]
#[clap(version = "1.0", about = "A simple CLI tool to organize files by type")]
struct Cli {
    /// Optional path to the directory to organize (Defaults to Downloads if not provided)
    #[clap(default_value = "")] // If no path provided, default to asking the user
    path: String,
}

#[derive(Debug, Error)]
enum FileOrganizerError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("WalkDir error")]
    WalkDir(#[from] walkdir::Error),
}

/// Displays a welcome message
fn greet_user() {
    let hello_yellow = format!(
        "{} {} {} {}",
        "Welcome".yellow(),
        "to the".blue(),
        "File Organizer".green(),
        "CLI tool!".red()
    );
    println!("{}", hello_yellow);
    println!("This tool will help you organize your files by moving them into folders based on their file types.");
    println!("Let's get started!\n");
}

/// Generates unique names for files to avoid overwriting
fn generate_unique_name(new_path: &PathBuf) -> PathBuf {
    let mut unique_path = new_path.clone();
    let mut counter = 1;
    while unique_path.exists() {
        let file_stem = new_path.file_stem().unwrap_or_else(|| OsStr::new(""));
        let extension = new_path
            .extension()
            .unwrap_or_else(|| OsStr::new(""))
            .to_string_lossy()
            .to_string();
        unique_path = new_path.with_file_name(format!(
            "{}_{}.{}",
            file_stem.to_string_lossy(),
            counter,
            extension
        ));
        counter += 1;
    }
    unique_path
}

/// Detects the user's Downloads folder
fn get_downloads_folder() -> Option<PathBuf> {
    download_dir()
}

/// Prompts the user for confirmation
fn prompt_user(message: &str) -> bool {
    println!("{}", message);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_lowercase() == "y"
}

/// Organizes files in the specified directory
fn organize_files(path: &PathBuf) -> Result<(), FileOrganizerError> {
    println!(
        "Organizing files in directory: {}",
        path.display().to_string().blue()
    );

    let entries: Vec<_> = WalkDir::new(&path)
        .into_iter()
        .filter_map(Result::ok)
        .collect();
    let total_files = entries.iter().filter(|e| e.file_type().is_file()).count();

    let progress_bar = ProgressBar::new(total_files as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("#>-"),
    );

    let mut files_moved = 0;
    let mut dirs_created = 0;

    // Common file types to organize
    let common_file_types = vec![
        ("images", vec!["jpg", "jpeg", "png", "gif", "bmp"]),
        ("documents", vec!["pdf", "doc", "docx", "txt"]),
        ("audio", vec!["mp3", "wav", "flac"]),
        ("video", vec!["mp4", "mkv", "avi"]),
        ("code", vec!["rs", "py", "js", "html", "css"]),
        ("archives", vec!["zip", "rar", "7z"]),
    ];

    for entry in entries {
        if entry.file_type().is_file() {
            let file_path = entry.path();
            if let Some(extension) = file_path.extension() {
                let extension = extension.to_string_lossy().to_string().to_lowercase();
                let mut new_dir = path.join("others");

                for (category, exts) in &common_file_types {
                    if exts.contains(&extension.as_str()) {
                        new_dir = path.join(category);
                        break;
                    }
                }

                if !new_dir.exists() {
                    fs::create_dir_all(&new_dir)?;
                    dirs_created += 1;
                }

                let new_path = generate_unique_name(&new_dir.join(file_path.file_name().unwrap()));
                fs::rename(file_path, new_path.clone())?;
                println!(
                    "Moved file: {} to {}",
                    file_path.display().to_string().green(),
                    new_path.display().to_string().cyan()
                );
                files_moved += 1;
            }
            progress_bar.inc(1);
        }
    }

    progress_bar.finish_with_message("All files organized successfully!");
    println!("Total files moved: {}", files_moved.to_string().yellow());
    println!("Total directories created: {}", dirs_created.to_string().yellow());

    Ok(())
}

fn main() -> Result<(), FileOrganizerError> {
    let args = Cli::parse();

    greet_user();

    let path = if !args.path.is_empty() {
        PathBuf::from(args.path)
    } else if let Some(downloads_path) = get_downloads_folder() {
        println!(
            "Detected Downloads Folder: {}",
            downloads_path.display().to_string().blue()
        );
        if prompt_user("Would you like to organize your Downloads folder? (y/n): ") {
            downloads_path
        } else {
            println!("Operation canceled. See ya later!");
            return Ok(());
        }
    } else {
        println!("Could not detect Downloads folder, please provide a path.");
        return Ok(());
    };

    organize_files(&path)
}