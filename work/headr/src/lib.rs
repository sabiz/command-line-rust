use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

pub fn run(config: Config) -> MyResult<()> {
    let files = config.files;
    for (i, filename) in files.iter().enumerate() {
        match open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(mut buf_read) => {
                if files.len() > 1 {
                    println!(
                        "{}==> {} <==",
                        if i > 0 { "\n" } else { "" },
                        if filename == "-" {
                            "standard input"
                        } else {
                            &filename
                        }
                    );
                }
                if let Some(bytes) = config.bytes {
                    let mut buf = vec![0; bytes];
                    if let Ok(size) = buf_read.read(&mut buf) {
                        if size > 0 {
                            print!("{}", String::from_utf8_lossy(&buf));
                        }
                    }
                } else {
                    (0..config.lines).for_each(|_| {
                        let mut line = String::new();
                        let size = buf_read.read_line(&mut line).unwrap();
                        if size <= 0 {
                            return;
                        }
                        print!("{}", line);
                    });
                }
            }
        }
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("headr")
        .version("0.1.0")
        .author("John Doe")
        .about("Rust head")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input File(s)")
                .default_value("-")
                .multiple(true),
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .value_name("LINES")
                .help("Number of lines")
                .default_value("10")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .value_name("BYTES")
                .help("Number of bytes")
                .conflicts_with("lines")
                .takes_value(true),
        )
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines: matches
            .value_of("lines")
            .map(parse_positive_int)
            .transpose()
            .map_err(|e| format!("illegal line count -- {}", e))?
            .unwrap(),
        bytes: matches
            .value_of("bytes")
            .map(parse_positive_int)
            .transpose()
            .map_err(|e| format!("illegal byte count -- {}", e))?,
    })
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse::<usize>() {
        Ok(v) if v > 0 => Ok(v),
        _ => Err(From::from(val)),
    }
}

#[test]
fn test_parse_positive_int() {
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
