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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, Write};

    #[test]
    fn read_lines_from_file_yields_content() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "alpha").unwrap();
        writeln!(tmp, "beta").unwrap();
        let path = tmp.path();
        let mut reader = read_lines(Some(path)).unwrap();
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        assert_eq!(buf, "alpha\n");
        buf.clear();
        reader.read_line(&mut buf).unwrap();
        assert_eq!(buf, "beta\n");
    }

    #[test]
    fn read_lines_missing_file_is_error() {
        let path = std::path::Path::new("/nonexistent/rustbox_io_test_path");
        let err = read_lines(Some(path)).err().expect("expected io error");
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }
}