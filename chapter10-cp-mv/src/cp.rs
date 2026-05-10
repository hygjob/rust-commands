// clap CLI, 단일·재귀 복사와 대상 경로 규칙 처리
use clap::Parser;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

/// Rust로 만든 cp 명령어
/// 파일을 복사합니다
#[derive(Parser)]
#[command(name = "cp")]
#[command(version, about = "Rust로 만든 cp 명령어")]
struct Cli {
    /// 디렉토리를 재귀적으로 복사합니다
    #[arg(short = 'r')]
    recursive: bool,

    /// 덮어쓰기 전에 확인합니다.
    #[arg(short = 'i')]
    interactive: bool,

    /// 복사과정을 자세히 출력합니다. 
    #[arg(short = 'v')]
    verbose: bool,

    /// 모든 경로 (마지막 인자가 대상, 나머지가 원본)
    #[arg(required = true, num_args = 2..)]
    paths: Vec<String>,
}

/// `-i`: stderr로 물어보고 y/yes일 때만 덮어쓰기 허용.
fn confirm_overwrite(path: &str) -> bool {
    eprint!("cp: '{}'를 덮어쓰시겠습니까? (y/n) ", path);
    io::stderr().flush().unwrap();

    let stdin = io::stdin();
    let mut answer = String::new();
    stdin.lock().read_line(&mut answer).unwrap();

    answer.trim().to_lowercase() == "y" || answer.trim().to_lowercase() == "yes"
}

/// 일반 파일 한 개를 `fs::copy`로 복사합니다.
fn copy_file(src: &Path, dst: &Path, cli: &Cli) -> io::Result<()> {
    if dst.exists() && cli.interactive {
        if !confirm_overwrite(&dst.display().to_string()) {
            return Ok(());
        }
    }

    fs::copy(src, dst)?;

    if cli.verbose {
        println!("{} -> {}", src.display(), dst.display());
    }

    Ok(())
}

/// 디렉터리면 하위까지 재귀; 파일이면 `copy_file`로 위임. `-r` 없이 디렉터리면 메시지 후 스킵.
fn copy_recursive(src: &Path, dst: &Path, cli: &Cli) -> io::Result<()> {
    if src.is_dir() {
        if !cli.recursive {
            eprintln!("cp: -r 옵션이 지정되지 않았습니다: {}", src.display());
            return Ok(());
        }

        fs::create_dir_all(dst)?;

        // 각 자식에 대해 동일 규칙으로 재귀
        for entry in fs::read_dir(src)?.filter_map(|e| e.ok()) {
            let src_path = entry.path();
            let file_name = entry.file_name();
            let dst_path = dst.join(&file_name);

            copy_recursive(&src_path, &dst_path, cli)?;
        }
    } else {
        copy_file(src, dst, cli)?;
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    // 1. 마지막 인자를 destination으로, 나머지를 sources로 분리
    let (sources, destination) = cli.paths.split_at(cli.paths.len() - 1);
    let dst_root = Path::new(&destination[0]);

    for src in sources {
        let src_path = Path::new(src);

        if !src_path.exists() {
            eprintln!("cp: {}: 그런 파일이나 디렉토리가 없습니다", src);
            continue;
        }

        // 2. 대상이 디렉토리인 경우와 파일인 경우의 경로 처리
        let final_dst = if dst_root.is_dir() {
            dst_root.join(src_path.file_name().unwrap())
        } else if sources.len() > 1 {
            // 원본이 여러 개인데 대상이 디렉토리가 아니면 에러
            eprintln!("cp: 대상 '{}'이 디렉토리가 아닙니다", dst_root.display());
            std::process::exit(1);
        } else {
            dst_root.to_path_buf()
        };

        if let Err(e) = copy_recursive(src_path, &final_dst, &cli) {
            eprintln!("cp: {}: {}", src, e);
        }
    }
}
