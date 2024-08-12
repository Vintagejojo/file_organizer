use clap::Parser;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;
use std::io;
use thiserror::Error;
use std::ffi::OsStr;

/// A simple CLI tool to organize files by type
#[derive(Parser)]
#[clap(version = "1.0", about = "A simple CLI tool to organize files by type")]
struct Cli {
    /// The path to the directory to organize
    path: PathBuf,
}

#[derive(Error, Debug)]
enum FileOrganizerError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("WalkDir error")]
    WalkDir(#[from] walkdir::Error),
}

fn greet_user() {
    println!("Welcome to the File Organizer CLI!");
    println!("This tool will help you organize your files by moving them into folders based on their file types.");
    println!("Let's get started!\n");
}

fn generate_unique_name(new_path: &PathBuf) -> PathBuf {
    let mut unique_path = new_path.clone();
    let mut counter = 1;
    while unique_path.exists() {
        let file_stem = new_path.file_stem().unwrap_or_else(|| OsStr::new(""));
        let extension = new_path.extension().unwrap_or_else(|| OsStr::new(""));
        unique_path = new_path.with_file_name(format!(
            "{}_{}.{}",
            file_stem.to_string_lossy(),
            counter,
            extension.to_string_lossy()
        ));
        counter += 1;
    }
    unique_path
}

fn main() -> Result<(), FileOrganizerError> {
    let args = Cli::parse();

    greet_user();

    println!("Organizing files in directory: {:?}", args.path);

    let mut files_moved = 0;
    let mut dirs_created = 0;

    for entry in WalkDir::new(&args.path).into_iter() {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        let extension = extension.to_string_lossy().to_string();
                        let new_dir = args.path.join(&extension);
                        if !new_dir.exists() {
                            fs::create_dir_all(&new_dir)?;
                            dirs_created += 1;
                        }
                        let new_path = generate_unique_name(&new_dir.join(path.file_name().unwrap()));
                        fs::rename(path, new_path.clone())?;
                        println!("Moved file: {:?} to {:?}", path, new_path);
                        files_moved += 1;
                    }
                }
            },
            Err(err) => {
                eprintln!("Failed to access entry: {}", err);
            }
        }
    }

    println!("Total files moved: {}", files_moved);
    println!("Total directories created: {}", dirs_created);
    println!("All files have been organized successfully!");
    Ok(())
}
