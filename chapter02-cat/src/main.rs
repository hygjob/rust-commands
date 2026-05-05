use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader, stdin, Write};

/// Rust로 만든 cat 명령어
/// 파일 내용을 표준 출력에 표시합니다
#[derive(Parser)]
#[command(name = "cat")]
#[command(version, about = "Rust로 만든 cat 명령어")]
struct Cli {
    /// 모든 줄에 번호를 매깁니다
    #[arg(short = 'n')]
    number: bool,

    /// 빈 줄이 아닌 줄에만 번호를 매깁니다
    #[arg(short = 'b')]
    number_nonblank: bool,

    /// 연속된 빈 줄을 하나로 압축합니다
    #[arg(short = 's')]
    squeeze_blank: bool,

    /// 출력할 파일 목록
    files: Vec<String>,
}

fn cat_lines<R: BufRead>(
    reader: &mut R,
    number: bool,
    number_nonblank: bool,
    squeeze_blank: bool,
) -> io::Result<()> {
    let mut line_number = 1;
    let mut prev_blank = false;

    for line in reader.lines() {
        let line = line?;

        if squeeze_blank && line.is_empty() {
            if prev_blank {
                continue;
            }
            prev_blank = true;
        } else {
            prev_blank = false;
        }

        if number_nonblank {
            if line.is_empty() {
                println!();
            } else {
                println!("{:6}\t{}", line_number, line);
                line_number += 1;
            }
        } else if number {
            println!("{:6}\t{}", line_number, line);
            line_number += 1;
        } else {
            println!("{}", line);
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if cli.files.is_empty() {
        let stdin = stdin();
        let mut reader = BufReader::new(stdin.lock());
        cat_lines(&mut reader, cli.number, cli.number_nonblank, cli.squeeze_blank)?;
    } else {
        for filename in &cli.files {
            match File::open(filename) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    cat_lines(&mut reader, cli.number, cli.number_nonblank, cli.squeeze_blank)?;
                }
                Err(e) => {
                    eprintln!("cat: {}: {}", filename, e);
                }
            }
        }
    }

    io::stdout().flush()?;
    Ok(())
}
