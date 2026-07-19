use clap::{Command, Arg, arg};
use serde::{ de::DeserializeOwned};
use serde_derive::Deserialize;
use std::error::Error;
use std::io::{BufReader};
use std::{ fmt::Debug};
use std::{fs::File};
use std::fs::remove_file;
use std::path::{Path, PathBuf, Component};
use std::cmp::Ordering;

/// Serializable struct containing entries for a specific file. These structs will identify individual files as a collection of paths and associated hash and length data.
#[derive(Debug, Deserialize)]
pub struct Fileinfo {
    // full_hash: Option<u128>,
    // partial_hash: Option<u128>,
    pub(crate) file_paths: Vec<PathBuf>,
}

#[repr(i8)]
enum SortOrder {
    Ascending,
    Descending
}

#[repr(i8)]
enum PathPrefixDupePreference {
    None,
    Short,
    Long
}

#[repr(i8)]
enum PathPrefixAlphaPreference {
    First,
    Last
}

static DDH_MOVE_RS_ABOUT: &str = "Read DDH JSON files and do something with the data contained in them.";

fn main(){
    let arguments = cli().get_matches();

    let input: Vec<_> = arguments.values_of("input").unwrap().collect();
    let mut dupe_files = get_dupe_files(input[0]).unwrap();

    match arguments.subcommand(){
        Some(("useprefix", sub_matches)) => {
            let pathprefix = sub_matches.value_of("PATHPREFIX").expect("required");
            let dupe_preference = 
                if sub_matches.is_present("prefershort") {
                    PathPrefixDupePreference::Short
                } else if sub_matches.is_present("preferlong") {
                    PathPrefixDupePreference::Long
                } else {
                    PathPrefixDupePreference::None
                };
                let blacklist = if sub_matches.is_present("blacklist") {
                    let vals: Vec<&str> = sub_matches.values_of("blacklist").unwrap().collect();
                    vals
                } else { let v: Vec<&str> = Vec::new(); v };
                println!("Blacklist: {0:?}", blacklist);
                let whitelist = if sub_matches.is_present("whitelist") {
                    let vals: Vec<&str> = sub_matches.values_of("whitelist").unwrap().collect();
                    vals
                } else { 
                    let v: Vec<&str> = Vec::new(); 
                    v 
                };
                println!("Whitelist: {0:?}", whitelist);
            keep_prefixed_file(pathprefix, &mut dupe_files, dupe_preference, whitelist, blacklist);
        }
        Some(("prefershort", _)) => { 
            sort_dupes_by_shorter_length(&mut dupe_files);
         }
        Some(("preferlong", _)) => { 
            sort_dupes_by_longer_length(&mut dupe_files);
         }
        Some(("prefersorted", sub_matches))  => { 
            let dupe_preference = 
                if sub_matches.is_present("first") {
                    PathPrefixAlphaPreference::First
                } else if sub_matches.is_present("last") {
                    PathPrefixAlphaPreference::Last
                } else {
                    PathPrefixAlphaPreference::First
                };
            sort_dupes_alphabetically(&mut dupe_files, dupe_preference);
        }
        None => { 
            // Do nothing special
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    };

    let mut deleted_files = Vec::new();
    for file in dupe_files.iter_mut() {
        let files_to_delete: Vec<_> = file.file_paths.iter().skip(1).collect();
        if arguments.is_present("dryrun") {
            println!("Dry run: For {0:?}, deleting {1:?}", file.file_paths[0], files_to_delete);
        } else {
            for path in files_to_delete {
                let path_to_delete = path.as_path();
                match remove_file(path_to_delete) {
                    Ok(_) => deleted_files.push(path_to_delete),
                    Err(e) => println!("Error deleting {0:?}: {1:?}", path_to_delete, e)
                }
            }
        }
    }

    println!("Done!");
    println!("Successfully deleted: {0:?}", deleted_files);
}

fn keep_prefixed_file(pathprefix: &str, dupe_files: &mut Vec<Fileinfo>, dupe_preference: PathPrefixDupePreference, whitelist: Vec<&str>, blacklist: Vec<&str>) {
    for file in dupe_files.iter_mut() {
        file.file_paths.sort_by(|a, b| {
            // Blacklist trumps everything
            if pathlist_contains_any_path_components(a, &blacklist) && pathlist_contains_any_path_components(b, &blacklist) {
                match dupe_preference {
                    PathPrefixDupePreference::None => a.cmp(b),
                    PathPrefixDupePreference::Short => pathbuf_len_sort(a, b, SortOrder::Ascending),
                    PathPrefixDupePreference::Long => pathbuf_len_sort(a, b, SortOrder::Descending)
                }
            } else if pathlist_contains_any_path_components(b, &blacklist) {
                Ordering::Less
            } else if pathlist_contains_any_path_components(a, &blacklist) {
                Ordering::Greater
            // Then check whitelist
            } else if pathlist_contains_any_path_components(a, &whitelist) && pathlist_contains_any_path_components(b, &whitelist) {
                match dupe_preference {
                    PathPrefixDupePreference::None => a.cmp(b),
                    PathPrefixDupePreference::Short => pathbuf_len_sort(a, b, SortOrder::Ascending),
                    PathPrefixDupePreference::Long => pathbuf_len_sort(a, b, SortOrder::Descending)
                }
            } else if pathlist_contains_any_path_components(a, &whitelist) {
                Ordering::Less
            } else if pathlist_contains_any_path_components(b, &whitelist) {
                Ordering::Greater
            } else if a.starts_with(pathprefix) && b.starts_with(pathprefix) {
                match dupe_preference {
                    PathPrefixDupePreference::None => a.cmp(b),
                    PathPrefixDupePreference::Short => pathbuf_len_sort(a, b, SortOrder::Ascending),
                    PathPrefixDupePreference::Long => pathbuf_len_sort(a, b, SortOrder::Descending)
                }
            } else if a.starts_with(pathprefix){
                Ordering::Less
            } else if b.starts_with(pathprefix){
                Ordering::Greater
            } else {
                match dupe_preference {
                    PathPrefixDupePreference::None => a.cmp(b),
                    PathPrefixDupePreference::Short => pathbuf_len_sort(a, b, SortOrder::Ascending),
                    PathPrefixDupePreference::Long => pathbuf_len_sort(a, b, SortOrder::Descending)
                }
            }
        });
    }
}

fn pathlist_contains_any_path_components(path: &PathBuf, list: &Vec<&str>) -> bool {
    list.iter().any(|c|{ path_contains_component(&path, c) })
}

fn path_contains_component(path: &PathBuf, target: &str) -> bool {
    // Iterate over components and compare as strings
    path.components().any(|comp| {
        // Only match normal path components (skip RootDir, CurDir, etc.)
        if let Component::Normal(os_str) = comp {
            os_str == target
        } else {
            false
        }
    })
}

fn path_parent_len(path: &Path) -> usize {
    match path.parent() {
        Some(parent) => parent.to_str().unwrap().len(),
        None => 0
    }
}

fn path_filename_len(path: &Path) -> usize {
    match path.file_name() {
        Some(file_name) => file_name.len(),
        None => 0
    }
}

fn pathbuf_len_sort(a: &PathBuf, b: &PathBuf, sort_order: SortOrder) -> Ordering {
    // We are intentionally swapping a and b, depending on the sort order
    let a_path = match sort_order {
        SortOrder::Ascending => a.as_path(),
        SortOrder::Descending => b.as_path()
    };
    let b_path = match sort_order {
        SortOrder::Ascending => b.as_path(),
        SortOrder::Descending => a.as_path()
    };

    let a_parent_len = path_parent_len(a_path);
    let b_parent_len = path_parent_len(b_path);

    match a_parent_len.cmp(&b_parent_len) {
        std::cmp::Ordering::Equal => {
            let a_file_name_len = path_filename_len(a_path);
            let b_file_name_len = path_filename_len(b_path);
            a_file_name_len.cmp(&b_file_name_len)
        },
        order => order
    }
}

fn sort_dupes_by_shorter_length(dupe_files: &mut Vec<Fileinfo>) {
    for file in dupe_files.iter_mut() {
        file.file_paths.sort_by(|a, b| pathbuf_len_sort(a,b,SortOrder::Ascending));
    }
}

fn sort_dupes_by_longer_length(dupe_files: &mut Vec<Fileinfo>) {
    for file in dupe_files.iter_mut() {
        file.file_paths.sort_by(|a, b| pathbuf_len_sort(a,b,SortOrder::Descending));
    }
}

fn sort_dupes_alphabetically(dupe_files: &mut Vec<Fileinfo>, dupe_preference: PathPrefixAlphaPreference) {
    match dupe_preference {
        PathPrefixAlphaPreference::First => {
            for file in dupe_files.iter_mut() {
                file.file_paths.sort_by(|a, b| a.cmp(&b));
            }
        }
        PathPrefixAlphaPreference::Last => {
            for file in dupe_files.iter_mut() {
                file.file_paths.sort_by(|a, b| b.cmp(&a));
            }
        }
    };
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
        .arg(Arg::new("dryrun")
                .short('n')
                .long("dryrun")
                .takes_value(false)
                .help("Do dry run (do not delete files, only print what we would do)"))
        .subcommand(
            Command::new("useprefix")
                .about("Prefer a path prefix when deciding what file to keep")
                .arg(arg!(<PATHPREFIX> "The prefix to prefer"))
                .arg_required_else_help(true)
                .arg(Arg::new("prefershort")
                        .short('s')
                        .long("prefershort")
                        .takes_value(false)
                        .conflicts_with("preferlong")
                        .help("When dupes are present, prefer the shorter one"))
                .arg(Arg::new("preferlong")
                        .short('l')
                        .long("preferlong")
                        .takes_value(false)
                        .conflicts_with("prefershort")
                        .help("When dupes are present, prefer the longer one"))
                .arg(Arg::new("blacklist")
                        .short('b')
                        .long("blacklist")
                        .takes_value(true)
                        .multiple_occurrences(true)
                        .help("Never prefer to keep paths containing this string"))
                .arg(Arg::new("whitelist")
                        .short('w')
                        .long("whitelist")
                        .takes_value(true)
                        .multiple_occurrences(true)
                        .help("Always prefer to keep paths containing this string"))
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
            Command::new("prefersorted")
            .about("Prefer an alphabetically sorted path when deciding what file to keep")
                .arg(Arg::new("first")
                        .short('a')
                        .long("first")
                        .takes_value(false)
                        .conflicts_with("last")
                        .help("Keep first alphabetical path"))
                .arg(Arg::new("last")
                        .short('z')
                        .long("last")
                        .takes_value(false)
                        .conflicts_with("first")
                        .help("Keep last alphabetical path"))
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

#[cfg(test)]
mod tests {
    use std::{path::{PathBuf}};
    use crate::{Fileinfo, keep_prefixed_file, pathlist_contains_any_path_components, path_contains_component};

    impl Fileinfo {
    #[inline]
        fn from(v: Vec<PathBuf>) -> Fileinfo {
            Fileinfo { file_paths: v }
        }
    }

    #[test]
    fn fileinfo_ctor_works_correctly() {
        let one = "one";
        let two = "two";
        let fi = Fileinfo::from(vec![PathBuf::from(one), PathBuf::from(two)]);
        assert_eq!(fi.file_paths[0].to_str().unwrap(), one);
        assert_eq!(fi.file_paths[1].to_str().unwrap(), two);
    }

    #[test]
    fn path_contains_component_works_correctly() {
        let test_path = PathBuf::from("foo/bar/bas");
        assert!(path_contains_component(&test_path, "foo"));
        assert!(path_contains_component(&test_path, "bar"));
        assert!(path_contains_component(&test_path, "bas"));
    }

    #[test]
    fn pathlist_contains_any_path_components_works_correctly() {
        let test_path = PathBuf::from("foo/bar/bas");

        let test_vec_1 = vec!["foo"];
        let test_vec_2 = vec!["blagh", "bar"];
        let test_vec_3 = vec!["foo", "bar", "bas"];
        let test_vec_4 = vec!["blagh"];

        assert!(pathlist_contains_any_path_components(&test_path, &test_vec_1));
        assert!(pathlist_contains_any_path_components(&test_path, &test_vec_2));
        assert!(pathlist_contains_any_path_components(&test_path, &test_vec_3));
        assert!(!pathlist_contains_any_path_components(&test_path, &test_vec_4));
    }
     
    #[test]
    fn paths_matching_pathprefix_are_first() {
        let path_prefix = "bar/";
        let dupe_file = Fileinfo::from(
            vec![
                PathBuf::from("foo/some-file.txt"), 
                PathBuf::from("bar/.hidden-folder/some-file2.txt"), 
                PathBuf::from("bar/some-file.txt"),
                PathBuf::from("bas/some-file.txt")
                ]);
        let mut dupes = vec![dupe_file];
        let whitelist = vec![];
        let blacklist: Vec<&str> = vec![];

        keep_prefixed_file(path_prefix, &mut dupes, crate::PathPrefixDupePreference::None, whitelist, blacklist);

        assert!(dupes[0].file_paths[0].starts_with(path_prefix));
    }

    #[test]
    fn user_can_blacklist_paths() {
        let path_prefix = "bar/";
        let dupe_file = Fileinfo::from(
            vec![
                PathBuf::from("foo/some-file.txt"), 
                PathBuf::from("bar/.hidden-folder/some-file2.txt"), 
                PathBuf::from("bar/some-file.txt"),
                PathBuf::from("bas/some-file.txt")
                ]);
        let mut dupes = vec![dupe_file];
        let test_path_component = ".hidden-folder";
        let whitelist = vec![];
        let blacklist: Vec<&str> = vec![test_path_component];

        keep_prefixed_file(path_prefix, &mut dupes, crate::PathPrefixDupePreference::None, whitelist, blacklist);

        assert!(!path_contains_component(&dupes[0].file_paths[0], test_path_component));
    }

    #[test]
    fn user_can_whitelist_paths() {
        let path_prefix = "bar/";
        let dupe_file = Fileinfo::from(
            vec![
                PathBuf::from("foo/some-file.txt"), 
                PathBuf::from("bar/.hidden-folder/some-file2.txt"), 
                PathBuf::from("bar/1/some-file.txt"),
                PathBuf::from("bar/2/some-file.txt"),
                PathBuf::from("bas/some-file.txt")
                ]);
        let mut dupes = vec![dupe_file];
        let test_path_component = "2"; // This path would ordinarily sort after "1"
        let whitelist = vec![test_path_component];
        let blacklist: Vec<&str> = vec![];

        keep_prefixed_file(path_prefix, &mut dupes, crate::PathPrefixDupePreference::None, whitelist, blacklist);

        assert!(path_contains_component(&dupes[0].file_paths[0], test_path_component));
    }

}
