use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use std::thread;

fn main() {
    // 1. 스피너 생성
    let pb = ProgressBar::new_spinner();
    
    // 2. 스타일 설정 (스피너의 모양과 메시지 위치 정의)
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            // 스피너가 사용할 문자 세트 정의 (기본값은 "-" " \ " "|" "/" 등)
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );

    pb.set_message("데이터 분석 중...");

    for _ in 0..100 {
        // 3. 핵심: tick()을 호출해야 스피너가 다음 모양으로 바뀝니다.
        pb.tick(); 
        
        // 실제 작업 수행 (예: 디렉토리 탐색 또는 계산)
        thread::sleep(Duration::from_millis(50));
    }

    // 4. 완료 처리
    pb.finish_with_message("작업 완료!");
}