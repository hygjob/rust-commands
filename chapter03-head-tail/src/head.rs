// clap CLI, 파일·stdin 읽기 및 stdout 출력
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, stdin, Write};
use std::path::PathBuf;

/// Rust로 만든 head 명령어
#[derive(Parser)]
#[command(name = "head")]
#[command(version, about = "Rust로 만든 head 명령어")]
struct Cli {
    /// 출력할 줄 수
    #[arg(short = 'n', default_value = "10")]
    lines: usize,

    /// 출력할 바이트 수
    #[arg(short = 'c')]
    bytes: Option<usize>,

    /// 입력 파일
    files: Vec<PathBuf>,
}

/// 앞에서부터 최대 `n`바이트만 표준 출력으로 복사합니다 (`-c`).
fn head_bytes<R: Read>(reader: &mut R, n: usize) -> io::Result<()> {
    let mut handle = reader.take(n as u64);
    let mut stdout = io::stdout();
    io::copy(&mut handle, &mut stdout)?;
    Ok(())
}

/// 앞에서부터 `n`줄만 출력합니다 (`-n`, 기본 10).
fn head_lines<R: BufRead>(reader: &mut R, n: usize) -> io::Result<()> {
    for (i, line) in reader.lines().enumerate() {
        if i >= n {
            break;
        }
        let line = line?;
        println!("{}", line);
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // 인자 없으면 stdin (`-c`면 바이트, 아니면 줄)
    if cli.files.is_empty() {
        let stdin = stdin();
        let mut reader = BufReader::new(stdin.lock());
        match cli.bytes {
            Some(n) => head_bytes(&mut reader, n)?,
            None => head_lines(&mut reader, cli.lines)?,
        }
    } else {
        // 여러 파일이면 구분용 헤더 출력 후 각각 head
        for (i, path) in cli.files.iter().enumerate() {
            if cli.files.len() > 1 {
                if i > 0 {
                    println!();
                }
                println!("==> {} <==", path.display());
            }

            match File::open(path) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    match cli.bytes {
                        Some(n) => head_bytes(&mut reader, n)?,
                        None => head_lines(&mut reader, cli.lines)?,
                    }
                }
                Err(e) => eprintln!("head: {}: {}", path.display(), e),
            }
        }
    }

    io::stdout().flush()?; // 버퍼 비우기
    Ok(())
}


// fn head_file(filename: &str, n: usize) -> io::Result<()> {
//     let file = File::open(filename)?;
//     let reader = BufReader::new(file);

//     for (i, line) in reader.lines().enumerate() {
//         if i >= n {
//             break;
//         }
//         let line = line?;
//         println!("{}", line);
//     }

//     Ok(())
// }

// fn main() -> io::Result<()> {
//     head_file("sample.txt", 3)
// }
