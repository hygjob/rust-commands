use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

fn copy_with_progress(src: &Path, dst: &Path) -> std::io::Result<()> {
    let src_file = File::open(src)?;
    let total_size = src_file.metadata()?.len();
    let mut dst_file = File::create(dst)?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::with_template(
        "{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"
    ).unwrap()
    .progress_chars("#>-"));
    pb.set_message(format!("복사: {}", src.display()));

    let mut reader = std::io::BufReader::new(src_file);
    let mut buffer = vec![0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        dst_file.write_all(&buffer[..bytes_read])?;
        pb.inc(bytes_read as u64);
    }

    pb.finish_with_message(format!("완료: {}", dst.display()));
    Ok(())
}

fn main() {
    let src = Path::new("large_source.dat"); // 존재하는 큰 파일 경로
    let dst = Path::new("large_copy.dat");

    if let Err(e) = copy_with_progress(src, dst) {
        eprintln!("오류 발생: {}", e);
    }
}