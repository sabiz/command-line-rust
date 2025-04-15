use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::{error::Error, f32::consts::E};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq, Eq)]
enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn run(config: Config) -> MyResult<()> {
    for path in config.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Err(e) => eprintln!("{}", e),
                Ok(entry) => {
                    println!("{}", entry.path().display())
                }
            }
        }
    }
    Ok(())
}


pub fn get_args() -> MyResult<Config> {
    let matches = App::new("uniqr")
    .version("0.1.0")
    .author("John Doe")
    .about("Rust find")
    .arg(
        Arg::with_name("paths")
        .value_name("PATH")
        .help("Search paths")
        .default_value(".")
        .multiple(true)
    )
    .arg(
        Arg::with_name("names")
        .value_name("NAME")
        .long("name")
        .short("n")
        .help("Name")
        .multiple(true)
        .takes_value(true)
    )
    .arg(
        Arg::with_name("entry_types")
        .value_name("TYPE")
        .long("type")
        .short("t")
        .help("Entry type")
        .multiple(true)
        .takes_value(true)
        .possible_values(&["f", "d", "l"])
    )
    .get_matches();

    Ok(Config {
        paths: matches.values_of_lossy("paths").unwrap(),
        names: matches.values_of_lossy("names").map(|v| {
            v.into_iter()
                .map(|s|Regex::new(&s).map_err(|_| format!("Invalid --name \"{}\"", s)))
                .collect::<Result<Vec<_>, _>>()
        }).transpose()?.unwrap_or_default(),
        entry_types: matches.values_of_lossy("types").map(|values| {
            values.iter().map(|v| match v.as_str() {
                "f" => File,
                "d" => Dir,
                "l" => Link,
                _ => unreachable!("Invalid type"),
            }).collect::<Vec<_>>()
        }).unwrap_or_default(),
    })
}