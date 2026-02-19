use crate::label::{ Label };

use indicatif::{ ProgressBar, ProgressStyle };
use rand::{ Rng };
use std::io::{ Write };
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

pub fn press_key_continue() {
    Label::PressKeyContinue.log();
    let mut input = String::new();
    let _ = std::io::stdout().flush();
    let _ = std::io::stdin().read_line(&mut input);
    clear_screen();
    let _ = std::io::stdout().flush();
}

pub async fn throttle() {
    tokio::time::sleep(tokio::time::Duration::from_secs(rand::rng().random_range(1..3))).await;
}

pub fn parse_date(mes_ano: &str) -> String {
    let date = mes_ano.trim();
    let parts: Vec<&str> = date.split('/').collect();
    if parts.len() != 2 {
        return "1900-01-01".to_string();
    }

    let month_num = match parts[0].to_lowercase().as_str() {
        "janeiro" => "01",
        "fevereiro" => "02",
        "março" => "03",
        "abril" => "04",
        "maio" => "05",
        "junho" => "06",
        "julho" => "07",
        "agosto" => "08",
        "setembro" => "09",
        "outubro" => "10",
        "novembro" => "11",
        "dezembro" => "12",
        _ => "01",
    };

    format!("{}-{}-01", parts[1], month_num)
}

pub fn progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template(
            "{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})"
        )
            .unwrap()
            .progress_chars("#>-")
    );
    pb
}
