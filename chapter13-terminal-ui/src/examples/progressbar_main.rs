use indicatif::{ProgressBar, ProgressStyle};

fn main() {
    let pb = ProgressBar::new(100);

    // let files_len: u64 = 100  ;    
    // let pb = ProgressBar::new(files_len); 
    // pb.set_style(
    //     ProgressStyle::with_template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
    //         .unwrap()
    //         .progress_chars("#>-"),
    // );

    for _ in 0..100 {
        pb.inc(1);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    pb.finish_with_message("완료");
}
