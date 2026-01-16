use anyhow::{Context, Result};
use clap::{Arg, ArgAction, Command};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> Result<()> {
    let mut cmd = Command::new("owo")
        .version("0.1.0")
        .author("xOphiuchus")
        .about("Like tree but outputs file contents to a single markdown file")
        .override_help(r#"USAGE:
    owo [OPTIONS] [PATH]

EXAMPLES:
    owo -o content.md
    owo -I "obj|bin|build|dist" -o content.md -w
    owo --help

FLAGS:
    -w, --with-dotfiles    Include hidden files and directories
    -h, --help             Print help information
    -V, --version          Print version information

OPTIONS:
    -I, --ignore <PATTERNS>    Ignore files/directories matching these patterns (pipe-separated) [default: obj|bin|build|dist|.git|.env|.env.*]
    -o, --output <FILE>        Output file (required)

ARGS:
    <PATH>                     Directory to traverse [default: current directory]"#)
        .arg(
            Arg::new("ignore")
                .short('I')
                .long("ignore")
                .value_name("PATTERNS")
                .help("Ignore files/directories matching these patterns (pipe-separated)")
                .default_value("obj|bin|build|dist|.git|.env|.env.*")
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file")
                .required(true)
        )
        .arg(
            Arg::new("with_dotfiles")
                .short('w')
                .long("with-dotfiles")
                .action(ArgAction::SetTrue)
                .help("Include dotfiles (hidden files and directories)")
        )
        .arg(
            Arg::new("directory")
                .help("Directory to traverse [default: current directory]")
                .default_value(".")
        )
        .disable_help_flag(true)
        .disable_version_flag(true);

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        println!("{}", cmd.render_help());
        return Ok(());
    }

    let matches = cmd.clone().try_get_matches().map_err(|e| {
        if e.kind() == clap::error::ErrorKind::DisplayHelp {
            println!("{}", cmd.render_help());
            std::process::exit(0);
        }
        e
    })?;

    let ignore_patterns = matches.get_one::<String>("ignore").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();
    let with_dotfiles = matches.get_flag("with_dotfiles");
    let directory = matches.get_one::<String>("directory").unwrap();

    let mut override_builder = OverrideBuilder::new(directory);
    for pattern in ignore_patterns.split('|') {
        let pattern = pattern.trim();
        if !pattern.is_empty() {
            override_builder.add(&format!("!{}", pattern))?;
        }
    }
    let overrides = override_builder.build()?;

    let walker = WalkBuilder::new(directory)
        .hidden(false)
        .git_ignore(true)
        .require_git(false)
        .overrides(overrides)
        .build()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if path.is_dir() {
                return Some(entry);
            }

            if !with_dotfiles {
                for component in path.components() {
                    if let std::path::Component::Normal(name) = component {
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with('.') && name_str != ".git" {
                            return None;
                        }
                    }
                }
            }

            Some(entry)
        })
        .collect::<Vec<_>>();

    let output = Arc::new(Mutex::new(String::new()));
    let semaphore = Arc::new(Semaphore::new(num_cpus::get() * 2));
    let mut handles = Vec::new();

    for entry in walker {
        if entry.path().is_file() {
            let path = entry.path().to_path_buf();
            let output = Arc::clone(&output);
            let semaphore = Arc::clone(&semaphore);

            let permit = semaphore.clone().acquire_owned().await?;
            handles.push(tokio::spawn(async move {
                let _permit = permit;
                if let Ok(content) = read_file_with_fallback(&path).await {
                    let mut output_lock = output.lock().unwrap();
                    output_lock.push_str(&format!("\n## File: `{}`\n", path.display()));
                    output_lock.push_str("```");
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        output_lock.push_str(ext);
                    }
                    output_lock.push_str("\n");
                    output_lock.push_str(&content.trim_end());
                    output_lock.push_str("\n```\n");
                }
            }));
        }
    }

    for handle in handles {
        let _ = handle.await;
    }

    let final_output = output.lock().unwrap().clone();
    fs::write(output_file, final_output)
        .await
        .context("Failed to write output file")?;

    println!("Successfully wrote output to {}", output_file);
    Ok(())
}

async fn read_file_with_fallback(path: &Path) -> Result<String> {
    match fs::read_to_string(path).await {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
            let bytes = fs::read(path).await?;
            Ok(format!("[Binary file: {} bytes]", bytes.len()))
        }
        Err(e) => Err(e.into()),
    }
}