use globset::{Glob, GlobSetBuilder};
use std::{env, path::Path};

use clap::Parser;
use walkdir::{DirEntry, WalkDir};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,

    #[arg(short, long, default_value_t = env::current_dir().unwrap().as_path().display().to_string())]
    directory: String,
}

fn main() {
    // let args = Args::parse();

    let mut builder = GlobSetBuilder::new();

    builder.add(Glob::new("**/node_modules").unwrap());
    builder.add(Glob::new("**/dist").unwrap());
    builder.add(Glob::new("**/.git").unwrap());
    let match_these_glob = builder.build().unwrap();

    let mut builder2 = GlobSetBuilder::new();

    builder2.add(Glob::new("**/node_modules/*").unwrap());
    builder2.add(Glob::new("**/dist/*").unwrap());
    builder2.add(Glob::new("**/.git/*").unwrap());
    let dont_match_glob = builder2.build().unwrap();

    // TODO: don't keep walking when in excluded directory or hidden directory
    let walker = WalkDir::new(".").into_iter();
    for entry in walker {
        let entry = entry.unwrap();
        let a = match_these_glob.matches(entry.path()).len();
        let b = dont_match_glob.matches(entry.path()).len();

        if a > 0 && b == 0 {
            println!("{:?}", entry.path());
        }
    }
}
