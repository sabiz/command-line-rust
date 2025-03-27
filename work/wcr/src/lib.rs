use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn run(config: Config) -> MyResult<()> {
    let mut total_info = FileInfo {
        num_lines: 0,
        num_words: 0,
        num_bytes: 0,
        num_chars: 0,
    };
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                let info = count(file)?;
                println!(
                    "{}{}",
                    format_info(&config, &info),
                    if filename != "-" {
                        format!(" {}", filename).to_string()
                    } else {
                        "".to_string()
                    }
                );
                total_info.num_lines += info.num_lines;
                total_info.num_words += info.num_words;
                total_info.num_bytes += info.num_bytes;
                total_info.num_chars += info.num_chars;
            }
        }
    }
    if config.files.len() > 1 {
        println!("{} total", format_info(&config, &total_info));
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("wcr")
        .version("0.1.0")
        .author("John Doe")
        .about("Rust wc")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input File(s)")
                .default_value("-")
                .multiple(true),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .help("Show byte count")
                .takes_value(false)
                .conflicts_with("chars"),
        )
        .arg(
            Arg::with_name("chars")
                .short("m")
                .long("chars")
                .help("Show character count")
                .takes_value(false)
                .conflicts_with("bytes"),
        )
        .arg(
            Arg::with_name("lines")
                .short("l")
                .long("lines")
                .help("Show line count")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("words")
                .short("w")
                .long("words")
                .help("Show word count")
                .takes_value(false),
        )
        .get_matches();

    let mut lines = matches.is_present("lines");
    let mut words = matches.is_present("words");
    let mut bytes = matches.is_present("bytes");
    let chars = matches.is_present("chars");

    if [lines, words, bytes, chars].iter().all(|v| v == &false) {
        lines = true;
        words = true;
        bytes = true;
    }

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines,
        words,
        bytes,
        chars,
    })
}

pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;
    let mut line = String::new();

    loop {
        let bytes_read = file.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }
        num_lines += 1;
        num_words += line.split_ascii_whitespace().count();
        num_bytes += bytes_read;
        num_chars += line.chars().count();
        line.clear();
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

fn format_info(config: &Config, info: &FileInfo) -> String {
    format!(
        "{}{}{}{}",
        format_count(&info.num_lines, config.lines),
        format_count(&info.num_words, config.words),
        format_count(&info.num_bytes, config.bytes),
        format_count(&info.num_chars, config.chars)
    )
    .to_string()
}

fn format_count(size: &usize, flg: bool) -> String {
    if flg {
        format!("{:>8}", size).to_string()
    } else {
        "".to_string()
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

#[cfg(test)]
mod tests {
    use super::{FileInfo, count};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_bytes: 48,
            num_chars: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }
}
