use clap::Parser;
use std::io::{self, BufRead, BufReader, stdin, Write};

/// Rust로 만든 tr 명령어
/// 문자를 변환하거나 삭제합니다
#[derive(Parser)]
#[command(name = "tr")]
#[command(version, about = "Rust로 만든 tr 명령어")]
struct Cli {
    /// 지정한 문자를 삭제합니다
    #[arg(short = 'd')]
    delete: bool,

    /// 변환할 문자 집합1
    set1: String,

    /// 변환할 문자 집합2 (-d 옵션에서는 생략 가능)
    set2: Option<String>,
}

fn parse_char_set(input: &str) -> Vec<char> {
    let mut chars = Vec::new();
    let input_chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < input_chars.len() {
        if i + 2 < input_chars.len() && input_chars[i + 1] == '-' {
            let start = input_chars[i];
            let end = input_chars[i + 2];
            if start <= end {
                for c in start..=end {
                    if !chars.contains(&c) {
                        chars.push(c);
                    }
                }
            }
            i += 3;
        } else {
            if !chars.contains(&input_chars[i]) {
                chars.push(input_chars[i]);
            }
            i += 1;
        }
    }

    chars
}

fn translate(line: &str, from: &[char], to: &[char]) -> String {
    line.chars().map(|c| {
        if let Some(idx) = from.iter().position(|&f| f == c) {
            if idx < to.len() {
                to[idx]
            } else {
                to[to.len() - 1]
            }
        } else {
            c
        }
    }).collect()
}

fn delete_chars(line: &str, chars: &[char]) -> String {
    line.chars().filter(|c| !chars.contains(c)).collect()
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let set1 = parse_char_set(&cli.set1);

    let stdin = stdin();
    let reader = BufReader::new(stdin.lock());

    for line in reader.lines() {
        let line = line?;

        let output = if cli.delete {
            delete_chars(&line, &set1)
        } else if let Some(ref s2) = cli.set2 {
            let set2 = parse_char_set(s2);
            translate(&line, &set1, &set2)
        } else {
            line
        };

        println!("{}", output);
    }

    io::stdout().flush()?;
    Ok(())
}
