use crate::ui::{ Label, Sql };
use crate::schema::Config;

use indicatif::{ ProgressBar, ProgressStyle };
use rand::{ Rng };
use rusqlite::{ params, Connection, Result };
use std::io::{ Write };
pub fn setup_db(conn: &Connection, recreate: bool) -> Result<(), Box<dyn std::error::Error>> {
    if recreate {
        let pb = progress_bar(11);
        // Drops
        conn.execute_batch(Sql::DropTables.as_str())?;
        pb.inc(1);
        // Config
        pb.set_message((Label::CreateTable { table_name: "config" }).to_string());
        conn.execute_batch(Sql::CreateConfig.as_str())?;
        pb.inc(1);
        // References
        pb.set_message((Label::CreateTable { table_name: "references" }).to_string());
        conn.execute_batch(Sql::CreateReferences.as_str())?;
        pb.inc(1);
        // Types
        pb.set_message((Label::CreateTable { table_name: "types" }).to_string());
        conn.execute_batch(Sql::CreateTypes.as_str())?;
        pb.inc(1);
        conn.execute(Sql::InitTypes.as_str(), ["Carros", "Motos", "Caminhões e Micro-Ônibus"])?;
        pb.inc(1);
        // Fuels
        pb.set_message((Label::CreateTable { table_name: "fuels" }).to_string());
        conn.execute_batch(Sql::CreateFuels.as_str())?;
        pb.inc(1);
        conn.execute(Sql::InitFuels.as_str(), [
            "Gasolina",
            "Álcool",
            "Diesel",
            "Gás Natural",
            "Flex",
            "Elétrico",
            "Híbrido",
            "Híbrido Plug-in",
        ])?;
        pb.inc(1);
        // Brands
        conn.execute_batch(Sql::CreateBrands.as_str())?;
        pb.inc(1);
        pb.set_message((Label::CreateTable { table_name: "brands" }).to_string());
        // Models
        pb.set_message((Label::CreateTable { table_name: "models" }).to_string());
        conn.execute_batch(Sql::CreateModels.as_str())?;
        pb.inc(1);
        // Years
        pb.set_message((Label::CreateTable { table_name: "years" }).to_string());
        conn.execute_batch(Sql::CreateYears.as_str())?;
        pb.inc(1);
        // Indexes
        pb.set_message(Label::CreateIndexes.to_string());
        conn.execute_batch(Sql::CreateIndexes.as_str())?;
        pb.inc(1);

        pb.finish_with_message(Label::DbCreationOk.to_string());
        Ok(())
    } else {
        Ok(())
    }
}

pub async fn select_status(conn: &Connection) -> Result<Config, Box<dyn std::error::Error>> {
    let mut stmt = match conn.prepare(&Sql::SelectConfig.as_str()) {
        Ok(s) => s,
        _ => {
            conn.execute_batch(&Sql::CreateConfig.as_str())?;
            return Ok(Config {
                db_status: "empty".to_string(),
                last_update: None,
            });
        }
    };

    let mut config_iter = stmt.query_map([], |row| {
        Ok(Config {
            db_status: row.get(0)?,
            last_update: row.get(1)?,
        })
    })?;

    if let Some(config) = config_iter.next() {
        Ok(config?)
    } else {
        Ok(Config {
            db_status: "empty".to_string(),
            last_update: None,
        })
    }
}

pub fn update_status(conn: &Connection, status: &str) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(&Sql::UpdateConfig.as_str(), params![status])?;
    Ok(())
}
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
