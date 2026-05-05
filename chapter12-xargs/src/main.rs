use clap::Parser;
use std::io::{self, BufRead, Read, Write};
use std::process::Command;

/// Rust로 만든 xargs 명령어
/// 표준 입력을 명령어 인자로 변환합니다
#[derive(Parser)]
#[command(name = "xargs")]
#[command(version, about = "Rust로 만든 xargs 명령어")]
struct Cli {
    /// 명령어당 최대 인자 수
    #[arg(short = 'n', long = "max-args")]
    max_args: Option<usize>,

    /// 입력 구분자 (기본값: 줄바꿈)
    #[arg(short = 'd', long = "delimiter")]
    delimiter: Option<String>,

    /// 실행 전에 확인합니다
    #[arg(short = 'p', long = "interactive")]
    interactive: bool,

    /// 입력이 없으면 실행하지 않습니다
    #[arg(short = 'r', long = "no-run-if-empty")]
    no_run_if_empty: bool,

    /// 실행할 명령어와 초기 인자들
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

fn read_input_args(delimiter: &Option<String>) -> Vec<String> {
    if let Some(delim) = delimiter {
        let mut input = String::new();
        io::stdin().lock().read_to_string(&mut input).unwrap();
        input.trim_end()
            .split(delim)
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        io::stdin().lock()
            .lines()
            .filter_map(|l| l.ok())
            .filter(|l| !l.is_empty())
            .collect()
    }
}

fn run_batches(
    cmd: &str,
    initial_args: &[String],
    input_args: &[String],
    max_args: Option<usize>,
    interactive: bool,
) -> i32 {
    let batch_size = max_args.unwrap_or(usize::MAX);
    let mut last_exit = 0i32;

    for chunk in input_args.chunks(batch_size) {
        let mut all_args = initial_args.to_vec();
        all_args.extend(chunk.iter().cloned());

        if interactive {
            eprint!("{} {}? (y/n) ", cmd, all_args.join(" "));
            io::stderr().flush().unwrap();

            let stdin = io::stdin();
            let mut answer = String::new();
            stdin.lock().read_line(&mut answer).unwrap();
            if answer.trim().to_lowercase() != "y" {
                continue;
            }
        }

        let status = match Command::new(cmd).args(&all_args).status() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("xargs: {}: {}", cmd, e);
                return 127;
            }
        };

        let code = status.code().unwrap_or(1);
        if code != 0 {
            last_exit = code;
        }
    }

    last_exit
}

fn main() {
    let cli = Cli::parse();

    let input_args = read_input_args(&cli.delimiter);

    if input_args.is_empty() && cli.no_run_if_empty {        
        return;
    }

    let (cmd, initial_args) = if cli.command.is_empty() {
        ("echo".to_string(), Vec::new())
    } else {
        let mut parts = cli.command.clone();
        let cmd = parts.remove(0);
        (cmd, parts)
    };

    let exit_code = run_batches(
        &cmd,
        &initial_args,
        &input_args,
        cli.max_args,
        cli.interactive,
    );

    std::process::exit(exit_code);
}
