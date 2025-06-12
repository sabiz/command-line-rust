use clap::{App, Arg};
use std::{
    cmp::Ordering::*,
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    file1: String,
    file2: String,
    show_col1: bool,
    show_col2: bool,
    show_col3: bool,
    insensitive: bool,
    delimiter: String,
}

pub fn run(config: Config) -> MyResult<()> {
    let file1 = &config.file1;
    let file2 = &config.file2;

    if file1 == "-" && file2 == "-" {
        return Err(From::from("Both input files cannot be STDIN (\"-\")"));
    }

    let _file1 = open(file1)?;
    let _file2 = open(file2)?;
    println!(
        "Processing files: {} and {}",
        file1,
        file2
    );

    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("commr")
        .version("0.1.0")
        .author("John Doe")
        .about("Rust comm")
        .arg(
            Arg::with_name("file1")
                .value_name("FILE1")
                .help("Input file 1")
                .required(true)
        )
        .arg(
            Arg::with_name("file2")
                .value_name("FILE2")
                .help("Input file 2")
                .required(true)
        )
        .arg(
            Arg::with_name("show_col1")
                .short("1")
                .help("Suppress printing of column 1")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("show_col2")
                .short("2")
                .help("Suppress printing of column 2")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("show_col3")
                .short("3")
                .help("Suppress printing of column 3")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .help("Case-insensitive comparison of lines")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("delimiter")
                .short("d")
                .long("ouput-delimiter")
                .value_name("DELIM")
                .help("Output delimiter")
                .default_value("\t")
        )
 
        .get_matches();

    Ok(Config {
        file1: matches.value_of("file1").unwrap().to_string(),
        file2: matches.value_of("file2").unwrap().to_string(),
        show_col1: !matches.is_present("show_col1"),
        show_col2: !matches.is_present("show_col2"),
        show_col3: !matches.is_present("show_col3"),
        insensitive: matches.is_present("insensitive"),
        delimiter: matches.value_of("delimiter").unwrap().to_string(),
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename)
                .map_err(|e| format!("{}: {}", filename, e))?,
        ))),
    }
}
