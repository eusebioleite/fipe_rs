use crate::label::{ Label };
use crate::schema::{ References };
use chrono::{ Datelike, NaiveDate, Utc };
use indicatif::{ ProgressBar, ProgressStyle };
use rand::{ Rng };
use std::io::{ Write };
use rand::seq::{ IndexedRandom };

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
            .progress_chars("=> ")
    );
    pb
}

pub fn parse_ref_date(reference: &References) -> String {
    let date = NaiveDate::parse_from_str(&reference.ref_date, "%Y-%m-%d").unwrap_or_else(|_|
        NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()
    );
    let mes = match date.month() {
        1 => "janeiro",
        2 => "fevereiro",
        3 => "março",
        4 => "abril",
        5 => "maio",
        6 => "junho",
        7 => "julho",
        8 => "agosto",
        9 => "setembro",
        10 => "outubro",
        11 => "novembro",
        12 => "dezembro",
        _ => unreachable!(),
    };
    let mes_ano = format!("{}/{}", mes, date.year());
    mes_ano
}

pub fn get_random_user_agent() -> &'static str {
    const AGENTS: &[&str] = &[
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Mobile/15E148 Safari/604.1",
    ];

    AGENTS.choose(&mut rand::rng()).unwrap_or(&AGENTS[0])
}
pub fn parse_year(y: &str) -> (String, Option<String>) {
    let mut parts = y.splitn(2, '-');
    let year_raw = parts.next().unwrap_or("").trim();
    let fuel_id = parts.next().map(|s| s.to_string());
    let year_str = if year_raw == "32000" {
        Utc::now().year().to_string()
    } else {
        year_raw.to_string()
    };
    let year_date = format!("{}-01-01", year_str);

    (year_date, fuel_id)
}
