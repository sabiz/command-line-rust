use crate::TakeValue::*;
use clap::{App, Arg};
use once_cell::sync::OnceCell;
use regex::Regex;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
};

static NUM_REGEX: OnceCell<Regex> = OnceCell::new();

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: TakeValue,
    bytes: Option<TakeValue>,
    quiet: bool,
}

pub fn run(config: Config) -> MyResult<()> {
    for (i, filename ) in config.files.iter().enumerate() {
        match File::open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(_) => {
                let (total_lines, total_bytes) = count_lines_bytes(filename)?;
                if !config.quiet && config.files.len() > 1 {
                    println!("==> {} <==", filename);
                }
                if let Some(num_bytes) = &config.bytes {
                    let _ = print_bytes(File::open(filename)?, num_bytes, total_bytes);
                } else {
                    let _ = print_lines(
                        BufReader::new(File::open(filename)?),
                        &config.lines,
                        total_lines,
                    );
                }
                if !config.quiet && config.files.len() > 1 && i < config.files.len() - 1 {
                    println!("");
                }
            }
        }
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("tailr")
        .version("0.1.0")
        .author("John Doe")
        .about("Rust tail")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .value_name("LINES")
                .help("Number of lines")
                .default_value("10"),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .value_name("BYTES")
                .conflicts_with("lines")
                .help("Number of bytes"),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Suppress headers"),
        )
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines: matches
            .value_of("lines")
            .map(parse_num)
            .transpose()
            .map_err(|e| format!("illegal line count -- {}", e))?
            .unwrap(),
        bytes: matches
            .value_of("bytes")
            .map(parse_num)
            .transpose()
            .map_err(|e| format!("illegal byte count -- {}", e))?,
        quiet: matches.is_present("quiet"),
    })
}

fn parse_num(s: &str) -> MyResult<TakeValue> {
    let num_re = NUM_REGEX.get_or_init(|| Regex::new(r"^((\+|-)?\d+)$").unwrap());
    if s == "+0" {
        Ok(PlusZero)
    } else if let Some(caps) = num_re.captures(s) {
        let sign = caps.get(2);
        let num = caps[1].parse::<i64>()?;
        if sign.is_some() {
            Ok(TakeNum(num))
        } else {
            Ok(TakeNum(num * -1))
        }
    } else {
        Err(s.to_string().into())
    }
}

fn count_lines_bytes(filename: &str) -> MyResult<(i64, i64)> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let total_lines = buf.lines().count() as i64;
    let total_bytes = std::fs::metadata(filename)?.len() as i64;

    Ok((total_lines, total_bytes))
}

fn print_lines(mut file: impl BufRead, num_lines: &TakeValue, total_lines: i64) -> MyResult<()> {
    let start_index = get_start_index(num_lines, total_lines).unwrap_or(-1);
    if start_index < 0 {
        return Ok(());
    }
    let mut line = String::new();
    let mut count = 0;
    while let Ok(read_size) = file.read_line(&mut line) {
        if read_size == 0 {
            break; // EOF
        }
        if count >= start_index {
            print!("{}", line);
        }
        count += 1;
        if count >= start_index + total_lines {
            break;
        }
        line.clear();
    }
    Ok(())
}

fn print_bytes<T: Read + Seek>(mut file: T, num_bytes: &TakeValue, total_bytes: i64) -> MyResult<()> {
    let start_index = get_start_index(num_bytes, total_bytes).unwrap_or(-1);
    if start_index < 0 {
        return Ok(());
    }
    file.seek(SeekFrom::Start(start_index as u64))?;
    let mut buffer = vec![0; (total_bytes - start_index).max(0) as usize];
    let bytes_read = file.read(&mut buffer)?;
    if bytes_read > 0 {
        print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
    }
    Ok(())
}

fn get_start_index(take_val: &TakeValue, total: i64) -> Option<i64> {
    if *take_val == TakeNum(0) || total == 0 {
        return None;
    }
    match take_val {
        PlusZero => Some(0),
        TakeNum(n) if *n > 0 => {
            if *n > total {
                None
            } else {
                Some(n - 1)
            }
        }
        TakeNum(n) if *n < 0 => {
            if total + n < 0 {
                Some(0)
            } else {
                Some(total + n)
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TakeValue::*,
        count_lines_bytes,
        get_start_index,
        parse_num,
    };

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));

        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }

    #[test]
    fn test_get_start_index() {
        // +0 from an empty file (0 lines/bytes) returns None
        assert_eq!(get_start_index(&PlusZero, 0), None);

        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));

        // Taking 0 lines/bytes returns None
        assert_eq!(get_start_index(&TakeNum(0), 1), None);

        // Taking any lines/bytes from an empty file returns None
        assert_eq!(get_start_index(&TakeNum(1), 0), None);

        // Taking more lines/bytes than is available returns None
        assert_eq!(get_start_index(&TakeNum(2), 1), None);

        // When starting line/byte is less than total lines/bytes,
        // return one less than starting number
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));

        // When starting line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));

        // When the starting line/byte is negative and more than the total,
        // return 0 to print the whole file
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }

    #[test]
    fn test_parse_num() {
        // All integers should be interpreted as negative numbers
        let res = parse_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // A leading "+" should result in a positive number
        let res = parse_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        // An explicit "-" value should result in a negative number
        let res = parse_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // Zero is zero
        let res = parse_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        // Plus zero is special
        let res = parse_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        // Test boundaries
        let res = parse_num(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = parse_num(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        // A floating-point value is invalid
        let res = parse_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        // Any non-integer string is invalid
        let res = parse_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }
}
