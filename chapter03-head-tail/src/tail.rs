use clap::Parser;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, stdin, Write};
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

fn tail_lines<R: BufRead>(reader: &mut R, n: usize) -> io::Result<()> {
    let mut buf: VecDeque<String> = VecDeque::with_capacity(n);

    for line in reader.lines() {
        let line = line?;
        if buf.len() == n {
            buf.pop_front();
        }
        buf.push_back(line);
    }

    for line in &buf {
        println!("{}", line);
    }

    Ok(())
}

fn tail_bytes_from_file(path: &PathBuf, n: usize) -> io::Result<()> {
    let mut file = File::open(path)?;
    let file_size = file.seek(SeekFrom::End(0))?;

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

    io::stdout().flush()?;
    Ok(())
}
