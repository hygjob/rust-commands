use colored::*;
use comfy_table::{Table, ContentArrangement};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;

    if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

fn process_files(files: &[&Path]) {
    println!("{}", "파일 처리 시작".green().bold());
    println!();

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(ProgressStyle::with_template(
        "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} 파일 ({eta})"
    ).unwrap()
    .progress_chars("#>-"));

    let mut results: Vec<(String, String, String)> = Vec::new();

    for file in files {
        // 실제 처리 느낌을 내기 위한 짧은 지연
        thread::sleep(Duration::from_millis(100));

        let name = file.display().to_string();
        
        // 파일 메타데이터 가져오기
        let size = fs::metadata(file)
            .map(|m| format_size(m.len()))
            .unwrap_or_else(|_| "?".to_string());
        
        let status = "OK".green().to_string();

        results.push((name, size, status));
        pb.inc(1);
    }

    pb.finish_and_clear();

    // 결과를 테이블로 출력
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        "파일",
        "크기",
        "상태",
    ]);

    for (name, size, status) in &results {
        table.add_row(vec![name, size, status]);
    }

    println!("{table}");
    println!();
    println!("{} {}개 파일 처리 완료", "✓".green(), files.len());
}

fn main() -> std::io::Result<()> {
    // 1. 테스트용 더미 파일들 생성
    let test_files = vec![
        "test_alpha.txt",
        "test_beta.log",
        "test_gamma.dat",
    ];

    println!("테스트 환경 준비 중...");
    for file in &test_files {
        // 100KB 정도의 더미 데이터 채우기
        fs::write(file, vec![0u8; 1024 * 100])?;
    }

    // 2. Path 참조 리스트 생성
    let paths: Vec<&Path> = test_files.iter().map(Path::new).collect();

    // 3. 파일 처리 함수 실행
    process_files(&paths);

    // 4. 테스트 파일 삭제 (정리)
    println!("테스트 파일 정리 중...");
    for file in test_files {
        let _ = fs::remove_file(file);
    }

    Ok(())
}