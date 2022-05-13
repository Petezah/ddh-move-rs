use clap::{Command, Arg, arg};
use serde::{ de::DeserializeOwned};
use serde_derive::Deserialize;
use std::error::Error;
use std::io::{BufReader};
use std::{ fmt::Debug};
use std::{fs::File};
use std::path::PathBuf;
use std::cmp::Ordering;

/// Serializable struct containing entries for a specific file. These structs will identify individual files as a collection of paths and associated hash and length data.
#[derive(Debug, Deserialize)]
pub struct Fileinfo {
    full_hash: Option<u128>,
    partial_hash: Option<u128>,
    pub(crate) file_paths: Vec<PathBuf>,
}

static DDH_MOVE_RS_ABOUT: &str = "Read DDH JSON files and do something with the data contained in them.";

fn main(){
    let arguments = cli().get_matches();

    let input: Vec<_> = arguments.values_of("input").unwrap().collect();
    let mut dupe_files = get_dupe_files(input[0]).unwrap();

    match arguments.subcommand(){
        Some(("useprefix", sub_matches)) => {
            let pathprefix = sub_matches.value_of("PATHPREFIX").expect("required");
            keep_prefixed_file(pathprefix, &mut dupe_files);
        }
        Some(("prefershort", _)) => { 
            keep_prefixed_file("", &mut dupe_files);
            // paths.sort_by(|a, b| a.to_str().unwrap().len().cmp(&b.to_str().unwrap().len()));
            // let result: Vec<_> = paths.iter().skip(1).collect();
            // result
         }
        Some(("preferlong", _)) => { 
            keep_prefixed_file("", &mut dupe_files);
            // paths.sort_by(|a, b| b.to_str().unwrap().len().cmp(&a.to_str().unwrap().len()));
            // let result: Vec<_> = paths.iter().skip(1).collect();
            // result
         }
        Some(("preferfirstsorted", _))  => { 
            keep_prefixed_file("", &mut dupe_files);
            // paths.sort_by(|a, b| b.cmp(a));
            // let result: Vec<_> = paths.iter().skip(1).collect();
            // result
        }
        None => { 
            keep_prefixed_file("", &mut dupe_files);
            // paths.sort_by(|a, b| b.cmp(a));
            // let result: Vec<_> = paths.iter().skip(0).collect();
            // result
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    };

    // for file in dupe_files.iter_mut() {
    //     //let paths: Vec<_> = file.file_paths.iter().filter(keep_filter).collect();
    //     let paths = keep_filter(&mut file.file_paths);
    //     println!("{0:?}", paths);
    // }
}

fn keep_prefixed_file(pathprefix: &str, dupe_files: &mut Vec<Fileinfo>) {
    for file in dupe_files.iter_mut() {
        file.file_paths.sort_by(|a, b| {
            if a.starts_with(pathprefix) && b.starts_with(pathprefix) {
                println!("{0:?} {1:?} 1", a, b);
                a.cmp(b)
            } else if a.starts_with(pathprefix){
                println!("{0:?} {1:?} 2", a, b);
                Ordering::Less
            } else if b.starts_with(pathprefix){
                println!("{0:?} {1:?} 3", a, b);
                Ordering::Greater
            } else {
                println!("{0:?} {1:?} 4", a, b);
                a.cmp(b)
            }
        });
        println!("{0:?}", file.file_paths);
    }
    //let result: Vec<_> = paths.iter().skip(1).collect();
    //result
}

fn cli() -> Command<'static> {
    Command::new("Directory Difference hTool File Mover / Remover")
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
        .subcommand(
            Command::new("useprefix")
                .about("Prefer a path prefix when deciding what file to keep")
                .arg(arg!(<PATHPREFIX> "The prefix to prefer"))
                .arg_required_else_help(true)
        )
        .subcommand(
            Command::new("prefershort")
                .about("Prefer the shortest path when deciding what file to keep")
        )
        .subcommand(
            Command::new("preferlong")
            .about("Prefer the longest path when deciding what file to keep")
        )
        .subcommand(
            Command::new("preferfirstsorted")
            .about("Prefer the first alphabetical path when deciding what file to keep")
        )
}

fn get_dupe_files(path: &str) -> Result<Vec<Fileinfo>, Box<dyn Error>>
{
    let files: Vec<Fileinfo> = read_object(path).unwrap();
    let dupe_files: Vec<Fileinfo> = files.into_iter().filter(|file| file.file_paths.len() > 1).collect();

    Ok(dupe_files)
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
