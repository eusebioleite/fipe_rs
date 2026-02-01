mod ui;
mod schema;
mod loads;
mod selects;
mod utils;

use loads::{ load_brands, load_models, load_references, load_years };
use ui::{ Label };
use utils::{ clear_screen, press_key_continue, setup_db, select_status, update_status };
use rusqlite::{ Connection, Result };
use owo_colors::OwoColorize;
use std::process::exit;
use std::io::{ self };

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("fipe_rs.db")?;
    loop {
        clear_screen();

        let config = select_status(&conn).await?;
        let db_status: String = match config.db_status.as_str() {
            "empty" => "Empty".to_string().italic().to_string(),
            "stable" => "Stable".to_string().bright_green().to_string(),
            "compromised" =>
                "Compromised"
                    .to_string()
                    .bold()
                    .bright_red()
                    .blink_fast()
                    .strikethrough()
                    .italic()
                    .to_string(),
            _ => "Unknown".to_string().bright_red().to_string(),
        };
        let last_update = config.last_update.unwrap_or_else(|| "Never".to_string());
        (Label::MenuOptions { db_status: &db_status, last_update: &last_update }).log();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice: usize = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                Label::InvalidInput.log();
                continue;
            }
        };
        drop(input);

        match choice {
            1 => {
                match setup_db(&conn, true) {
                    Ok(_) => {
                        Label::DbCreationOk.log();
                        update_status(&conn, "stable")?;
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        (Label::DbCreationErr { message: &err_msg }).log();
                        update_status(&conn, "compromised")?;
                    }
                }
                press_key_continue();
            }
            2 => {
                load_references(&conn).await?;
                press_key_continue();
            }
            3 => {
                load_brands(&conn).await?;
                press_key_continue();
            }
            4 => {
                load_models(&conn).await?;
                press_key_continue();
            }
            5 => {
                load_years(&conn).await?;
                press_key_continue();
            }
            9 => {
                load_references(&conn).await?;
                load_brands(&conn).await?;
                load_models(&conn).await?;
                load_years(&conn).await?;
                press_key_continue();
            }
            0 => {
                exit(0);
            }
            _ => {
                Label::InvalidInput.log();
                press_key_continue();
            }
        }
    }
}
