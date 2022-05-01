use clap::{Command, Arg};
use serde::{ de::DeserializeOwned};
use serde_derive::Deserialize;
use std::error::Error;
use std::io::{BufReader};
use std::{ fmt::Debug};
use std::{fs::File};
use std::path::PathBuf;

/// Serializable struct containing entries for a specific file. These structs will identify individual files as a collection of paths and associated hash and length data.
#[derive(Debug, Deserialize)]
pub struct Fileinfo {
    full_hash: Option<u128>,
    partial_hash: Option<u128>,
    pub(crate) file_paths: Vec<PathBuf>,
}

static DDH_MOVE_RS_ABOUT: &str = "Read DDH JSON files and do something with the data contained in them.";

fn main(){
    let arguments = Command::new("Directory Difference hTool File Mover / Remover")
                        .version(env!("CARGO_PKG_VERSION"))
                        .author(env!("CARGO_PKG_AUTHORS"))
                        .about(DDH_MOVE_RS_ABOUT)
                        .arg(Arg::new("input")
                                .short('i')
                                .long("input")
                                .value_name("Input")
                                .help("Input JSON file")
                                .max_values(1)
                                .required(true)
                                .takes_value(true))
                        .get_matches();

    let input: Vec<_> = arguments.values_of("input").unwrap().collect();
    let files: Vec<Fileinfo> = read_object(input[0]).unwrap();

    for file in files.iter() {
        match file {
            Fileinfo { full_hash: _, partial_hash:  _, file_paths } if file_paths.len() > 1 => println!("{0:?}", file_paths),
            _ => {}
        }
    }
}


fn read_object<T>(path: &str) -> Result<T, Box<dyn Error>>
where
    T: DeserializeOwned + Debug,
{
    let f = File::open(&path)?;
    let reader = BufReader::new(f);

    let t: T = serde_json::from_reader(reader)?;

    Ok(t)
}
