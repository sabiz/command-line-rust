use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;
use std::{error::Error, fmt::format, fs::File, io::{BufRead, BufReader}, path::PathBuf};

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
    let entries = find_files(&config.files, config.recursive);
    for entry in entries {
        match entry {
            Err(e)=> eprintln!("{}", e),
            Ok(filename) => match open(&filename) {
                Err(e) => eprintln!("{}: {}", filename, e),
                Ok(file) => {
                    let matches = find_lines(file, &config.pattern, config.invert_match);
                    match matches {
                        Err(e) => eprintln!("{}: {}", filename, e),
                        Ok(lines) => {
                            if config.count {
                                if &config.files.len() > &1 || config.recursive {
                                    print!("{}:", filename);
                                }
                                println!("{}", lines.len());
                            } else {
                                for line in lines {
                                    if &config.files.len() > &1 || config.recursive {
                                       print!("{}:", filename);
                                    }
                                    print!("{}", line);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
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

fn find_files(path: &[String], recursive: bool) -> Vec<MyResult<String>> {
    path.iter().map(|p| {
        if p == "-" {
            return vec![Ok("-".to_string())];
        }
        let path = PathBuf::from(p);
        if !path.exists() {
            return vec![Err(format!("{}: does not exist", p).into())];
        }
        if path.is_file() {
            return vec![Ok(p.clone())];
        }
        if path.is_dir() {
            if !recursive {
                return vec![Err(format!("{} is a directory", p).into())];
            }
            let dirs:Vec<_> = WalkDir::new(path).min_depth(1).into_iter().filter_map(Result::ok).map(|p| {p.path().to_string_lossy().to_string()}).collect();
            return find_files(&dirs, true);
        }
        vec![Ok(p.clone())]
    }).flatten().collect()
}

fn find_lines<T: BufRead> (mut file: T, pattern: &Regex, invert_match: bool) -> MyResult<Vec<String>> {
    let mut matches = vec![];
    let mut line = String::new();

    while file.read_line(&mut line)? > 0 {
        if pattern.is_match(&line) != invert_match {
            matches.push(std::mem::take(&mut line));
        }
        line.clear();
    }
    Ok(matches)
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?)))
    }
}

#[cfg(test)]
mod tests {
    use super::{find_files, find_lines};
    use rand::{distributions::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";

        // The pattern _or_ should match the one line, "Lorem"
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);

        // When inverted, the function should match the other two lines
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // This regex will be case-insensitive
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();

        // The two lines "Lorem" and "DOLOR" should match
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // When inverted, the one remaining line should match
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let files =
            find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );

        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }
}
