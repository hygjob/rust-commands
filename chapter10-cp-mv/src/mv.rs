use clap::Parser;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

/// Rust로 만든 mv 명령어
/// 파일을 이동하거나 이름을 바꿉니다
#[derive(Parser)]
#[command(name = "mv")]
#[command(version, about = "Rust로 만든 mv 명령어")]
struct Cli {
    /// 덮어쓰기 전에 확인합니다
    #[arg(short = 'i')]
    interactive: bool,

    /// 이동 과정을 자세히 출력합니다
    #[arg(short = 'v')]
    verbose: bool,

    /// 모든 경로 (마지막 인자가 대상, 나머지가 원본)
    #[arg(required = true, num_args = 2..)]
    paths: Vec<String>,
}

fn confirm_overwrite(path: &str) -> bool {
    eprint!("mv: '{}'를 덮어쓰시겠습니까? (y/n) ", path);
    io::stderr().flush().unwrap();

    let stdin = io::stdin();
    let mut answer = String::new();
    stdin.lock().read_line(&mut answer).unwrap();

    answer.trim().to_lowercase() == "y" || answer.trim().to_lowercase() == "yes"
}

fn move_item(src: &Path, dst: &Path, cli: &Cli) -> io::Result<()> {
    if dst.exists() && cli.interactive {
        if !confirm_overwrite(&dst.display().to_string()) {
            return Ok(());
        }
    }

    // 같은 파일 시스템이면 rename으로 즉시 이동
    match fs::rename(src, dst) {
        Ok(_) => {
            if cli.verbose {
                println!("{} -> {}", src.display(), dst.display());
            }
            Ok(())
        }
        Err(_) => {
            // 다른 파일 시스템이면 copy + delete
            if src.is_dir() {
                copy_dir_recursive(src, dst)?;
                fs::remove_dir_all(src)?;
            } else {
                fs::copy(src, dst)?;
                fs::remove_file(src)?;
            }
            if cli.verbose {
                println!("{} -> {}", src.display(), dst.display());
            }
            Ok(())
        }
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)?.filter_map(|e| e.ok()) {
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    // 1. 마지막 인자를 destination으로, 나머지를 sources로 분리
    // cli.paths는 최소 2개 이상의 인자를 가짐 (clap 설정: num_args = 2..)
    let (sources, destinations) = cli.paths.split_at(cli.paths.len() - 1);
    let dst_root = Path::new(&destinations[0]);

    // 2. 다중 원본 이동 시 대상 디렉토리 체크
    // 여러 파일을 한꺼번에 이동하려면 대상이 반드시 디렉토리여야 함
    if sources.len() > 1 && !dst_root.is_dir() {
        eprintln!("mv: 대상 '{}'이 디렉토리가 아닙니다", dst_root.display());
        std::process::exit(1);
    }

    for src in sources {
        let src_path = Path::new(src);

        if !src_path.exists() {
            eprintln!("mv: {}: 그런 파일이나 디렉토리가 없습니다", src);
            continue;
        }

        // 3. 최종 목적지 결정
        // 대상이 디렉토리면 그 안에 원본 이름을 붙여서 이동, 아니면 대상 경로 그대로 사용
        let final_dst = if dst_root.is_dir() {
            dst_root.join(src_path.file_name().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "유효하지 않은 파일 이름")
            }).unwrap())
        } else {
            dst_root.to_path_buf()
        };

        // 4. 이동 실행
        if let Err(e) = move_item(src_path, &final_dst, &cli) {
            eprintln!("mv: '{}'를 '{}'로 이동 중 오류 발생: {}", src, final_dst.display(), e);
        }
    }
}
