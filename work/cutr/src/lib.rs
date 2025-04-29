use crate::Extract::*;
use clap::{App, Arg};
use core::num;
use std::{cmp::max, default, error::Error, ops::Range};

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
    println!("{:#?}", &config);
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
                .takes_value(true),
        )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .long("chars")
                .short("c")
                .help("Selected characters")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("fields")
                .value_name("FIELDS")
                .long("fields")
                .short("f")
                .help("Selected fields")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("delimiter")
                .value_name("DELIMITER")
                .long("delim")
                .short("d")
                .help("Field delimiter")
                .default_value(" ")
                .takes_value(true),
        )
        .get_matches();

    Ok(Config {
        files: matches
            .values_of("file")
            .unwrap()
            .map(|s| s.to_string())
            .collect(),
        delimiter: b'\t',
        extract: Extract::Fields(vec![0..1]),
    })
}

fn parse_number(value: &str) -> MyResult<usize> {
    match value.parse::<usize>() {
        Ok(num) => {
            if num == 0 {
                return Err("".into());
            }
            Ok(num)
        }
        Err(_) => {
            return Err("".into());
        }
        
    }
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    println!("---\nrange: {}", range);
    if range.is_empty() {
        return Err("empty string".into());
    }
    
    let mut result:PositionList = vec![];
    for value in range.split(',').map(|s| s.trim()) {
        println!("value: {}", value);
        if value.is_empty() || value.contains(|c: char| !c.is_numeric() && c != '-') {
            return Err(format!("illegal list value: \"{}\"", value).into());
        }

        let separated_value = value.split('-').collect::<Vec<_>>();
        if separated_value.len() > 2
            || separated_value[0].starts_with('+')
            || (separated_value.len() > 1 && separated_value[1].starts_with('+'))
        {
            return Err(format!("illegal list value: \"{}\"", value).into());
        }

        println!("separated_value: {:?}", separated_value);
        let mut start_end = vec![0,0];
        if separated_value.len() == 1 {
            let number_value = parse_number(separated_value[0]).map_err(|_| {
                format!("illegal list value: \"{}\"", separated_value[0])
            })?;
            start_end[0] = number_value;
            start_end[1] = number_value;
        }else if separated_value.len() == 2 {
            for (i, v) in separated_value.iter().enumerate() {
                let number_value = parse_number(v).map_err(|_| {
                    format!("illegal list value: \"{}\"", v)
                })?;
                start_end[i] = number_value;
            }
        }
        println!("start_end: {:?}", start_end);
        if separated_value.len() == 2 && start_end[0] >= start_end[1] {
            return Err(format!(
                "First number in range ({}) must be lower than second number ({})",
                start_end[0], start_end[1]
            )
            .into());
        }
        start_end[0] -= 1;
        println!("result start_end: {:?}", start_end);
        result.push(start_end[0]..start_end[1]);
    }
    Ok(result)
}

#[cfg(test)]
mod unit_tests {
    use super::parse_pos;

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
}
