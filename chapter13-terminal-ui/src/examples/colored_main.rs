use colored::*;

fn main() {
    // 색상
    println!("{}", "빨간색".red());
    println!("{}", "초록색".green());
    println!("{}", "파란색".blue());
    println!("{}", "노란색".yellow());
    println!("{}", "하늘색".cyan());
    println!("{}", "자주색".magenta());

    // 스타일
    println!("{}", "굵게".bold());
    println!("{}", "밑줄".underline());
    println!("{}", "기울임".italic());
    println!("{}", "깜빡임".blink());

    // 조합
    println!("{}", "빨간색 굵게".red().bold());
    println!("{}", "초록색 밑줄".green().underline());

    // 배경색
    println!("{}", "배경 노란색".on_yellow().black());
    println!("{}", "배경 파란색".on_blue().white());
}
