use colored::*;
use std::io::{self, IsTerminal};

fn main() {
    // 터미널인지 확인
    if io::stdout().is_terminal() {
        // colored는 기본적으로 터미널이 아니면 컬러를 비활성화합니다
        println!("{}", "컬러 출력".green().bold());
    } else {
        println!("일반 출력");
    }

    // 수동 제어
    colored::control::set_override(false);  // 컬러 끄기
    println!("{}", "컬러 꺼짐".red());  // 일반 텍스트로 출력

    colored::control::set_override(true);   // 컬러 켜기
    println!("{}", "컬러 켜짐".green());  // 컬러로 출력
}
