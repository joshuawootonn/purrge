use std::{error::Error, path::PathBuf};

use clap::{Parser, ValueHint};

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

pub struct Configuration {
    pub directory: PathBuf,
}

pub fn get_configuration() -> Configuration {
    let args = Args::parse();
    let directory = input_from_either(args.directory, args.directory_pos).unwrap();
    Configuration { directory }
}
