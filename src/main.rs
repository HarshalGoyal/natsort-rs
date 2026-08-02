//! CLI interface for natsort-rs
//!
//! This module provides the command-line interface that mimics Python's natsort.__main__.py

use std::io::{self, Read};

use natsort::{natsorted_with, realsorted, NsFlags};

/// CLI arguments structure
#[derive(Debug)]
pub struct CliArgs {
    pub files: Vec<String>,
    pub ignore_case: bool,
    pub reverse: bool,
    pub real: bool,
    pub help: bool,
}

/// Parse command line arguments
pub fn parse_args() -> CliArgs {
    let mut args = std::env::args().skip(1);
    let mut files = Vec::new();
    let mut ignore_case = false;
    let mut reverse = false;
    let mut real = false;
    let mut help = false;
    
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                help = true;
            }
            "-i" | "--ignore-case" => {
                ignore_case = true;
            }
            "-r" | "--reverse" => {
                reverse = true;
            }
            "-f" | "--real" => {
                real = true;
            }
            _ => {
                if arg.starts_with('-') {
                    eprintln!("Unknown option: {}", arg);
                    std::process::exit(1);
                }
                files.push(arg);
            }
        }
    }
    
    CliArgs {
        files,
        ignore_case,
        reverse,
        real,
        help,
    }
}

/// Print usage information
pub fn print_usage() {
    println!("Usage: natsort [OPTIONS] [FILES...]");
    println!("Options:");
    println!("  -h, --help            Display this help message");
    println!("  -i, --ignore-case     Case insensitive sorting");
    println!("  -r, --reverse         Reverse sort order");
    println!("  -f, --real            Sort as real numbers");
    println!();
    println!("If no files are specified, reads from stdin.");
}

/// Main CLI entry point
pub fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();
    
    if args.help {
        print_usage();
        return Ok(());
    }
    
    let flags = if args.ignore_case {
        NsFlags::IGNORECASE
    } else {
        NsFlags::default()
    };
    
    if args.files.is_empty() {
        // Read from stdin
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        let lines: Vec<&str> = input.lines().collect();
        
        let sorted = if args.real {
            realsorted(&lines)
        } else {
            natsorted_with(&lines, flags)
        };
        
        for line in sorted {
            println!("{}", line);
        }
    } else {
        // Convert Vec<String> to Vec<&str> for the functions
        let files: Vec<&str> = args.files.iter().map(|s| s.as_str()).collect();
        
        // Sort files directly
        let sorted = if args.real {
            realsorted(&files)
        } else {
            natsorted_with(&files, flags)
        };
        
        for file in sorted {
            println!("{}", file);
        }
    }
    
    Ok(())
}

fn main() {
    if let Err(e) = run_cli() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}