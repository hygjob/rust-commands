// crates/lib/src/io.rs

use std::fs::File;
use std::io::{self, BufRead, BufReader, stdin};
use std::path::Path;

/// 파일 또는 표준 입력에서 줄을 읽습니다
pub fn read_lines(path: Option<&Path>) -> io::Result<Box<dyn BufRead>> {
    match path {
        Some(p) => {
            let file = File::open(p)?;
            Ok(Box::new(BufReader::new(file)))
        }
        None => {
            let stdin = stdin();
            Ok(Box::new(BufReader::new(stdin.lock())))
        }
    }
}

/// 에러 메시지를 출력하고 종료합니다
pub fn exit_with_error(cmd: &str, msg: &str) -> ! {
    eprintln!("rustbox {}: {}", cmd, msg);
    std::process::exit(1)
}