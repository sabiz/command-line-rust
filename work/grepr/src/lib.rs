use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
        let matches = App::new("grepr")
        .version("0.1.0")
        .author("John Doe")
        .about("Rust grep")
        .arg(
            Arg::with_name("pattern")
            .value_name("PATTERN")
            .help("Search pattern")
            .required(true)
        )
        .arg(
            Arg::with_name("file")
            .value_name("FILE")
            .help("Input file(s)")
            .default_value("-")
            .multiple(true)
        )
        .arg(
            Arg::with_name("count")
            .short("c")
            .long("count")
            .help("Count occurrences")
            .takes_value(false)
        )
        .arg(
            Arg::with_name("insensitive")
            .short("i")
            .long("insensitive")
            .help("Case-insensitive")
            .takes_value(false)
        )
        .arg(
            Arg::with_name("invert-match")
            .short("v")
            .long("invert-match")
            .help("Invert match")
            .takes_value(false)
        )
        .arg(
            Arg::with_name("recursive")
            .short("r")
            .long("recursive")
            .help("Recursive search")
            .takes_value(false)
        )

        .get_matches();

    let pattern = RegexBuilder::new(matches.value_of("pattern").unwrap())
        .case_insensitive(matches.is_present("insensitive"))
        .build().map_err(|_| format!("Invalid pattern \"{}\"\n", matches.value_of("pattern").unwrap()))?;

    Ok(Config {
        pattern: pattern,
        files: matches.values_of("file").unwrap().map(|s| s.to_string()).collect(),
        recursive: matches.is_present("recursive"),
        count: matches.is_present("count"),
        invert_match: matches.is_present("invert-match"),
    })
}