use globset::{Glob, GlobSetBuilder};
use std::{env, error::Error, path::PathBuf};

use clap::{Parser, ValueHint};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        id = "directory",   
        help = "The directory which to search",
        long,
        short = 'd',
        conflicts_with = "directory_pos",
        required_unless_present = "directory_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub directory: Option<PathBuf>,

    #[arg(
        id = "directory_pos",
        help = "The directory to search - (positional)",
        conflicts_with = "directory",
        required_unless_present = "directory",
        value_hint = ValueHint::FilePath
    )]
    pub directory_pos: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let input = input_from_either(args.directory, args.directory_pos).unwrap();

    println!("input: {:?}", input);

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
    let walker = WalkDir::new(input.to_str().unwrap()).into_iter();
    for entry in walker {
        let entry = entry.unwrap();
        let a = match_these_glob.matches(entry.path()).len();
        let b = dont_match_glob.matches(entry.path()).len();

        if a > 0 && b == 0 {
            println!("{:?}", entry.path());
        }
    }
}

pub fn input_from_either(
    path_a: Option<PathBuf>,
    path_b: Option<PathBuf>,
) -> Result<PathBuf, Box<dyn Error>> {
    match path_a {
        Some(path_a) => Ok(path_a),
        None => match path_b {
            Some(path_b) => Ok(path_b),
            None => panic!("No input file provided. See `purrge --help`"),
        },
    }
}
