use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file)
        .map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut out_file = open_out(&config.out_file)?;
    let mut line = String::new();
    let mut last_line = String::new();
    let mut count = 1;
    loop {
        let bytes = file.read_line(&mut line)?;
        
        if line.trim_end_matches("\n") == last_line.trim_end_matches("\n") && bytes > 0 {
            count +=1;
            line.clear();
            continue;
        }   
        if !last_line.is_empty() {
            if config.count {
                write!(out_file, "{:>4} ", count)?;
            }
            write!(out_file, "{}",last_line)?;
        }
        if bytes == 0 {
            break;
        }
        last_line = line.clone();
        count = 1;
        line.clear();
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("uniqr")
        .version("0.1.0")
        .author("John Doe")
        .about("Rust uniq")
        .arg(
            Arg::with_name("in_file")
                .value_name("IN_FILE")
                .help("Input file")
                .default_value("-")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("out_file")
            .value_name("OUT_FILE")
            .help("Output file")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("count")
            .short("c")
            .long("count")
            .help("Show counts")
        )

        .get_matches();

    

    Ok(Config {
        in_file: matches.value_of("in_file").unwrap().to_string(),
        out_file: matches.value_of("out_file").map(String::from),
        count: matches.is_present("count"),
    })
}


fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn open_out(filename: &Option<String>) -> MyResult<Box<dyn io::Write>> {
    match filename {
        Some(name) => Ok(Box::new(File::create(name)?)),
        None => Ok(Box::new(io::stdout())),
    }

}