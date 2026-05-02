use clap::Parser;
use std::io::{self, Write};

/// Rust로 만든 echo 명령어
#[derive(Parser)]
#[command(name = "echo")]
#[command(version, about = "텍스트를 표준 출력에 표시합니다")]
struct Cli {
    /// 마지막에 줄바꿈을 하지 않습니다
    #[arg(short = 'n')]
    no_newline: bool,

    /// 백슬래시 이스케이프 해석을 활성화합니다
    #[arg(short = 'e')]
    escape: bool,

    /// 출력할 텍스트
    text: Vec<String>,
}

fn process_escapes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some('n') => { result.push('\n'); chars.next(); }
                Some('t') => { result.push('\t'); chars.next(); }
                Some('r') => { result.push('\r'); chars.next(); }
                Some('\\') => { result.push('\\'); chars.next(); }
                Some('a') => { result.push('\x07'); chars.next(); }
                Some('b') => { result.push('\x08'); chars.next(); }
                _ => result.push(c),
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn main() {
    let cli = Cli::parse();

    let text = cli.text.join(" ");
    let output = if cli.escape {
        process_escapes(&text)
    } else {
        text
    };

    if cli.no_newline {
        print!("{}", output);
    } else {
        println!("{}", output);
    }

    io::stdout().flush().unwrap();
}
