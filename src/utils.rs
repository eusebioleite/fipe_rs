use crate::ui::{ Label, Sql };
use crate::schema::Config;

use rusqlite::{ params, Connection, Result };
use std::io::{ self };
pub fn setup_db(conn: &Connection, recreate: bool) -> Result<(), Box<dyn std::error::Error>> {
    if recreate {
        // Years
        (Label::CreateTable { table_name: "years" }).log();
        conn.execute_batch(Sql::CreateYears.as_str())?;
        // Models
        (Label::CreateTable { table_name: "models" }).log();
        conn.execute_batch(Sql::CreateModels.as_str())?;
        // Brands
        (Label::CreateTable { table_name: "brands" }).log();
        conn.execute_batch(Sql::CreateBrands.as_str())?;
        // References
        (Label::CreateTable { table_name: "references" }).log();
        conn.execute_batch(Sql::CreateReferences.as_str())?;
        // Types
        (Label::CreateTable { table_name: "types" }).log();
        conn.execute_batch(Sql::CreateTypes.as_str())?;
        conn.execute(Sql::InitTypes.as_str(), ["Carros", "Motos", "Caminhões e Micro-Ônibus"])?;
        // Errors
        (Label::CreateTable { table_name: "errors" }).log();
        conn.execute_batch(Sql::CreateErrors.as_str())?;
        // Config
        (Label::CreateTable { table_name: "config" }).log();
        conn.execute_batch(Sql::CreateConfig.as_str())?;

        Ok(())
    } else {
        Ok(())
    }
}

// Types

// Years
pub async fn insert_error(
    conn: &Connection,
    entity: &str,
    url: &str,
    body: &str
) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(&Sql::InsertError.as_str(), params![entity, url, body])?;
    Ok(())
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
    let _ = io::stdin().read_line(&mut input);
}
