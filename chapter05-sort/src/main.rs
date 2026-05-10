// clap CLI, 줄 단위 읽기 후 비교·정렬
use clap::Parser;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{self, BufRead, BufReader, stdin};
use std::path::PathBuf;

/// Rust로 만든 sort 명령어
#[derive(Parser)]
#[command(name = "sort")]
#[command(version, about = "Rust로 만든 sort 명령어")]
struct Cli {
    /// 역순으로 정렬합니다
    #[arg(short = 'r')]
    reverse: bool,

    /// 숫자 기준으로 정렬합니다
    #[arg(short = 'n')]
    numeric: bool,

    /// 대소문자를 구분하지 않습니다
    #[arg(short = 'f')]
    ignore_case: bool,

    /// 중복 줄을 제거합니다
    #[arg(short = 'u')]
    unique: bool,

    /// 정렬 기준 필드 번호 (1부터 시작)
    #[arg(short = 'k')]
    key: Option<usize>,

    /// 입력 파일
    files: Vec<PathBuf>,
}

/// 공백 트림 후 `f64`로 파싱해 비교합니다. 둘 다 숫자가 아니면 문자열 사전순으로 대체합니다.
fn numeric_compare(a: &str, b: &str) -> Ordering {
    let num_a = a.trim().parse::<f64>();
    let num_b = b.trim().parse::<f64>();

    match (num_a, num_b) {
        (Ok(a), Ok(b)) => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => a.cmp(b),
    }
}

/// `-k`: 공백으로 나눈 필드 중 `field_num`번째(1부터)를 정렬 키로 씁니다.
fn get_field(line: &str, field_num: usize) -> String {
    let fields: Vec<&str> = line.split_whitespace().collect();
    fields.get(field_num - 1).unwrap_or(&"").to_string()
}

/// stdin 또는 여러 파일에서 줄을 모두 읽어 하나의 벡터로 합칩니다.
fn read_lines(paths: &[PathBuf]) -> io::Result<Vec<String>> {
    let mut lines = Vec::new();

    // 인자 없으면 표준 입력만
    if paths.is_empty() {
        let stdin = stdin();
        let reader = BufReader::new(stdin.lock());
        for line in reader.lines() {
            lines.push(line?);
        }
    } else {
        // 파일별로 순차적으로 줄 추가 (열기 실패는 해당 파일만 메시지)
        for path in paths {
            match File::open(path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    for line in reader.lines() {
                        lines.push(line?);
                    }
                }
                Err(e) => eprintln!("sort: {}: {}", path.display(), e),
            }
        }
    }

    Ok(lines)
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let mut lines = read_lines(&cli.files)?;

    // `-k`면 해당 필드만으로 비교, `-n`/`-f`/기본 문자열 비교와 조합; `-r`은 결과 순서 뒤집기
    lines.sort_by(|a, b| {
        let ord = if let Some(k) = cli.key {
            let key_a = get_field(a, k);
            let key_b = get_field(b, k);
            if cli.numeric {
                numeric_compare(&key_a, &key_b)
            } else {
                key_a.cmp(&key_b)
            }
        } else if cli.numeric {
            numeric_compare(a, b)
        } else if cli.ignore_case {
            a.to_lowercase().cmp(&b.to_lowercase())
        } else {
            a.cmp(b)
        };

        if cli.reverse { ord.reverse() } else { ord }
    });

    // `-u`: 정렬 후 인접한 동일 줄 제거 (정렬되어 있으므로 같은 줄은 모두 인접)
    if cli.unique {
        lines.dedup();
    }

    for line in &lines {
        println!("{}", line);
    }

    Ok(())
}
