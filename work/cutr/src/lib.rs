use crate::Extract::*;
use clap::{App, Arg};
use csv::{ReaderBuilder, StringRecord};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    num::NonZeroUsize,
    ops::Range,
};

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8,
    extract: Extract,
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(reader) => {
                match &config.extract {
                    Chars(pos) => {
                        reader.lines().for_each(|line| {
                            let line = line.unwrap();
                            let result = extract_chars(&line, pos);
                            println!("{}", result);
                        });
                    }
                    Bytes(pos) => {
                        reader.lines().for_each(|line| {
                            let line = line.unwrap();
                            let result = extract_bytes(&line, pos);
                            println!("{}", result);
                        });
                    }
                    Fields(pos) => {
                        let delimiter_str = String::from_utf8_lossy(&[config.delimiter.clone()]).to_string();
                        let mut reader = ReaderBuilder::new()
                            .has_headers(false)
                            .delimiter(config.delimiter)
                            .from_reader(reader);
                        reader.records().for_each(|line| {
                            let line = line.unwrap();
                            let result = extract_fields(&line, pos);
                            println!("{}", result.join(&delimiter_str));
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("cutr")
        .version("0.1.0")
        .author("John Doe")
        .about("Rust cut")
        .arg(
            Arg::with_name("file")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value("-")
                .multiple(true),
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .long("bytes")
                .short("b")
                .help("Selected bytes")
                .takes_value(true)
                .conflicts_with_all(&["fields", "chars"]),
        )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .long("chars")
                .short("c")
                .help("Selected characters")
                .takes_value(true)
                .conflicts_with_all(&["fields", "bytes"]),
        )
        .arg(
            Arg::with_name("fields")
                .value_name("FIELDS")
                .long("fields")
                .short("f")
                .help("Selected fields")
                .takes_value(true)
                .conflicts_with_all(&["bytes", "chars"]),
        )
        .arg(
            Arg::with_name("delimiter")
                .value_name("DELIMITER")
                .long("delim")
                .short("d")
                .help("Field delimiter")
                .default_value("\t")
                .takes_value(true),
        )
        .get_matches();

    let delimiter = matches.value_of("delimiter").unwrap();
    let delimiter = if delimiter.len() == 1 {
        delimiter.as_bytes()[0]
    } else {
        return Err(format!("--delim \"{}\" must be a single byte", delimiter).into());
    };

    let extract = if let Some(field) = matches.value_of("fields") {
        let fields = parse_pos(field)?;
        Extract::Fields(fields)
    } else if let Some(bytes) = matches.value_of("bytes") {
        let bytes = parse_pos(bytes)?;
        Extract::Bytes(bytes)
    } else if let Some(chars) = matches.value_of("chars") {
        let chars = parse_pos(chars)?;
        Extract::Chars(chars)
    } else {
        return Err("Must have --fields, --bytes, or --chars".into());
    };

    Ok(Config {
        files: matches
            .values_of("file")
            .unwrap()
            .map(|s| s.to_string())
            .collect(),
        delimiter,
        extract,
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    char_pos
        .iter()
        .map(|pos| {
            line.chars()
                .enumerate()
                .filter(|(i, _)| pos.contains(i))
                .map(|(_, c)| c)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("")
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    byte_pos
        .iter()
        .map(|pos| {
            String::from_utf8_lossy(
                line.as_bytes()
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| pos.contains(i))
                    .map(|(_, b)| b.clone())
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
            .to_string()
        })
        .collect::<Vec<_>>()
        .join("")
}

fn extract_fields(record: &StringRecord, field_pos: &[Range<usize>]) -> Vec<String> {
    field_pos
        .iter()
        .map(|pos| {
            record
                .iter()
                .enumerate()
                .filter(|(i, _)| pos.contains(i))
                .map(|(_, f)| f.to_string())
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>()
}

fn parse_number(value: &str) -> MyResult<usize> {
    match value.parse::<NonZeroUsize>() {
        Ok(n) => Ok(n.into()),
        _ => Err(format!("illegal list value: \"{}\"", value).into()),
    }
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    if range.is_empty() {
        return Err("empty position list".into());
    }

    let mut result = Vec::new();

    for part in range.split(',').map(str::trim) {
        if part.is_empty() {
            return Err("empty field in position list".into());
        }

        // 範囲指定かどうかを確認
        match part.split('-').collect::<Vec<_>>().as_slice() {
            // 単一の数値の場合
            [single] => {
                // 数値以外が含まれていないか確認
                if single.chars().any(|c| !c.is_ascii_digit()) {
                    return Err(format!("illegal list value: \"{}\"", part).into());
                }

                let num = parse_number(single)?;
                result.push((num - 1)..num);
            }
            // 範囲指定の場合
            [start, end] => {
                // 数値以外が含まれていないか確認
                if start.chars().any(|c| !c.is_ascii_digit())
                    || end.chars().any(|c| !c.is_ascii_digit())
                {
                    return Err(format!("illegal list value: \"{}\"", part).into());
                }

                let start_num = parse_number(start)?;
                let end_num = parse_number(end)?;

                if start_num >= end_num {
                    return Err(format!(
                        "First number in range ({}) must be lower than second number ({})",
                        start_num, end_num
                    )
                    .into());
                }

                result.push((start_num - 1)..end_num);
            }
            // その他の場合（不正な書式）
            _ => {
                return Err(format!("illegal list value: \"{}\"", part).into());
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod unit_tests {
    use super::{extract_bytes, extract_chars, extract_fields, parse_pos};
    use csv::StringRecord;

    #[test]
    fn test_parse_pos() {
        //空文字列はエラー
        assert!(parse_pos("").is_err());

        //ゼロはエラー
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        // 数字の前に「＋」が付く場合はエラー
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"",);

        let rest = parse_pos("+1-2");
        assert!(rest.is_err());
        assert_eq!(
            rest.unwrap_err().to_string(),
            "illegal list value: \"+1-2\"",
        );

        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-+2\"",);

        // 数字以外はエラー
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);

        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);

        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-a\"",);

        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"",);

        // エラーになる範囲
        let res = parse_pos("-");
        assert!(res.is_err());

        let res = parse_pos(",");
        assert!(res.is_err());

        let res = parse_pos("1,");
        assert!(res.is_err());

        let res = parse_pos("1-");
        assert!(res.is_err());

        let res = parse_pos("1-1-1");
        assert!(res.is_err());

        let res = parse_pos("1-1-a");
        assert!(res.is_err());

        //最初の数字は２番目より小さい必要がある
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)",
        );

        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)",
        );

        // 以下のケースは受け入れられる
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15, 19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }
}
