// clap CLI, stdin을 토큰으로 나눠 배치 실행·종료 코드 전달
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

/// `-d`가 있으면 전체 stdin을 구분자로 split, 없으면 줄 단위(빈 줄 제외).
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

/// `input_args`를 `-n` 크기(기본 무제한)로 잘라 매번 `cmd`에 붙여 실행합니다. 비영 코드는 마지막 것을 유지합니다.
fn run_batches(
    cmd: &str,
    initial_args: &[String],
    input_args: &[String],
    max_args: Option<usize>,
    interactive: bool,
) -> i32 {
    let batch_size = max_args.unwrap_or(usize::MAX);
    let mut last_exit = 0i32;

    // 각 배치: 고정 인자 + 이번 청크 인자
    for chunk in input_args.chunks(batch_size) {
        let mut all_args = initial_args.to_vec();
        all_args.extend(chunk.iter().cloned());

        // `-p`: 실행 직전 한 번씩 확인 (y가 아니면 해당 배치 스킵)
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
            last_exit = code; // 마지막 비정상 종료 코드를 반환값으로 누적
        }
    }

    last_exit
}

fn main() {
    let cli = Cli::parse();

    let input_args = read_input_args(&cli.delimiter);

    // `-r`: stdin 인자가 없으면 아무 것도 실행하지 않음
    if input_args.is_empty() && cli.no_run_if_empty {        
        return;
    }

    // 트레일링 인자: 첫 토큰이 실행 파일, 나머지는 매 배치 앞에 붙는 고정 인자
    let (cmd, initial_args) = if cli.command.is_empty() {
        // 인자 없으면 기본 명령은 `echo`
        ("echo".to_string(), Vec::new())
    } else {
        let mut parts = cli.command.clone();
        let cmd = parts.remove(0);
        (cmd, parts)
    };

    // 하위 프로세스 종료 코드를 그대로 프로세스 종료에 사용
    let exit_code = run_batches(
        &cmd,
        &initial_args,
        &input_args,
        cli.max_args,
        cli.interactive,
    );

    std::process::exit(exit_code);
}
