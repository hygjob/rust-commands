// clap CLI, 슬라이딩 버퍼·파일 seek로 끝부분 출력
use clap::Parser;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, stdin, Write};
use std::path::PathBuf;

/// Rust로 만든 tail 명령어
#[derive(Parser)]
#[command(name = "tail")]
#[command(version, about = "Rust로 만든 tail 명령어")]
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

/// 마지막 `n`줄만 유지하는 고정 크기 큐로 스트림을 한 번 순회합니다 (`-n`).
fn tail_lines<R: BufRead>(reader: &mut R, n: usize) -> io::Result<()> {
    let mut buf: VecDeque<String> = VecDeque::with_capacity(n);

    for line in reader.lines() {
        let line = line?;
        if buf.len() == n {
            buf.pop_front(); // 가장 오래된 줄 제거
        }
        buf.push_back(line);
    }

    for line in &buf {
        println!("{}", line);
    }

    Ok(())
}

/// 파일 끝에서부터 최대 `n`바이트를 읽습니다 (`-c`, seek 가능한 파일 전용).
fn tail_bytes_from_file(path: &PathBuf, n: usize) -> io::Result<()> {
    let mut file = File::open(path)?;
    let file_size = file.seek(SeekFrom::End(0))?;

    // 파일이 `n`바이트보다 작으면 처음부터 전체 복사
    let start = if file_size > n as u64 {
        file_size - n as u64
    } else {
        0
    };

    file.seek(SeekFrom::Start(start))?;
    let mut stdout = io::stdout();
    io::copy(&mut file, &mut stdout)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // 표준 입력은 줄 단위 tail만 지원 (바이트 모드 `-c` 없음)
    if cli.files.is_empty() {
        let stdin = stdin();
        let mut reader = BufReader::new(stdin.lock());
        tail_lines(&mut reader, cli.lines)?;
    } else {
        for (i, path) in cli.files.iter().enumerate() {
            if cli.files.len() > 1 {
                if i > 0 {
                    println!();
                }
                println!("==> {} <==", path.display());
            }

            // `-c`: 파일 끝에서 n바이트; 아니면 줄 단위 tail
            match cli.bytes {
                Some(n) => {
                    match File::open(path) {
                        Ok(_) => tail_bytes_from_file(path, n)?,
                        Err(e) => eprintln!("tail: {}: {}", path.display(), e),
                    }
                }
                None => {
                    match File::open(path) {
                        Ok(file) => {
                            let mut reader = BufReader::new(file);
                            tail_lines(&mut reader, cli.lines)?;
                        }
                        Err(e) => eprintln!("tail: {}: {}", path.display(), e),
                    }
                }
            }
        }
    }

    io::stdout().flush()?; // 버퍼 비우기
    Ok(())
}
