// clap으로 CLI 인자 파싱, 표준 입출력·파일 읽기
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

/// 버퍼 단위로 줄을 읽어 표준 출력으로 보냅니다. `-n`/`-b`/`-s` 조합을 처리합니다.
fn cat_lines<R: BufRead>(
    reader: &mut R,
    number: bool,
    number_nonblank: bool,
    squeeze_blank: bool,
) -> io::Result<()> {
    let mut line_number = 1;
    // `-s`: 직전 줄이 빈 줄이었는지 추적해 연속 빈 줄은 건너뜀
    let mut prev_blank = false;

    for line in reader.lines() {
        let line = line?;

        if squeeze_blank && line.is_empty() {
            if prev_blank {
                continue; // 두 번째 이후 연속 빈 줄은 출력하지 않음
            }
            prev_blank = true;
        } else {
            prev_blank = false;
        }

        // `-b`가 있으면 빈 줄은 번호 없이, 내용 있는 줄만 번호 증가
        if number_nonblank {
            if line.is_empty() {
                println!();
            } else {
                println!("{:6}\t{}", line_number, line);
                line_number += 1;
            }
        } else if number {
            // `-n`: 모든 줄에 번호 (빈 줄 포함)
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

    // 인자 없으면 표준 입력만 처리 (유닉스 cat과 동일)
    if cli.files.is_empty() {
        let stdin = stdin();
        let mut reader = BufReader::new(stdin.lock());
        cat_lines(&mut reader, cli.number, cli.number_nonblank, cli.squeeze_blank)?;
    } else {
        // 파일별로 순차 출력; 열기 실패는 해당 파일만 에러 메시지 후 계속
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

    // 버퍼링된 stdout 사용 시 남은 내용 확실히 출력
    io::stdout().flush()?;
    Ok(())
}
