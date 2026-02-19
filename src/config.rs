use crate::label::Label;
use crate::schema::{Count, RowCount, Status};
use crate::sql::Sql;
use crate::utils::progress_bar;

use rusqlite::{params, Connection, Name, Result};
pub fn setup_db(conn: &Connection, recreate: bool) -> Result<(), Box<dyn std::error::Error>> {
    if recreate {
        let pb = progress_bar(11);
        // Drops
        conn.execute_batch(Sql::DropTables.get().as_str())?;
        pb.inc(1);
        // Config
        pb.set_message(
            (Label::CreateTable {
                table_name: "config",
            })
            .to_string(),
        );
        conn.execute_batch(Sql::CreateConfig.get().as_str())?;
        pb.inc(1);
        // References
        pb.set_message(
            (Label::CreateTable {
                table_name: "references",
            })
            .to_string(),
        );
        conn.execute_batch(Sql::CreateReferences.get().as_str())?;
        pb.inc(1);
        // Types
        pb.set_message(
            (Label::CreateTable {
                table_name: "types",
            })
            .to_string(),
        );
        conn.execute_batch(Sql::CreateTypes.get().as_str())?;
        pb.inc(1);
        conn.execute(
            Sql::InitTypes.get().as_str(),
            ["Carros", "Motos", "Caminhões e Micro-Ônibus"],
        )?;
        pb.inc(1);
        // Fuels
        pb.set_message(
            (Label::CreateTable {
                table_name: "fuels",
            })
            .to_string(),
        );
        conn.execute_batch(Sql::CreateFuels.get().as_str())?;
        pb.inc(1);
        conn.execute(
            Sql::InitFuels.get().as_str(),
            [
                "Gasolina",
                "Álcool",
                "Diesel",
                "Gás Natural",
                "Flex",
                "Elétrico",
                "Híbrido",
                "Híbrido Plug-in",
            ],
        )?;
        pb.inc(1);
        // Brands
        conn.execute_batch(Sql::CreateBrands.get().as_str())?;
        pb.inc(1);
        pb.set_message(
            (Label::CreateTable {
                table_name: "brands",
            })
            .to_string(),
        );
        // Models
        pb.set_message(
            (Label::CreateTable {
                table_name: "models",
            })
            .to_string(),
        );
        conn.execute_batch(Sql::CreateModels.get().as_str())?;
        pb.inc(1);
        // Years
        pb.set_message(
            (Label::CreateTable {
                table_name: "years",
            })
            .to_string(),
        );
        conn.execute_batch(Sql::CreateYears.get().as_str())?;
        pb.inc(1);
        // Indexes
        pb.set_message(Label::CreateIndexes.to_string());
        conn.execute_batch(Sql::CreateIndexes.get().as_str())?;
        pb.inc(1);

        pb.finish_with_message(Label::DbCreationOk.to_string());
        Ok(())
    } else {
        Ok(())
    }
}

pub fn select_status(conn: &Connection) -> Result<Status, Box<dyn std::error::Error>> {
    let mut stmt = match conn.prepare(&Sql::SelectStatus.get().as_str()) {
        Ok(s) => s,
        _ => {
            conn.execute_batch(&Sql::CreateConfig.get().as_str())?;
            return Ok(Status {
                db_status: "empty".to_string(),
                last_update: None,
            });
        }
    };

    let mut config_iter = stmt.query_map([], |row| {
        Ok(Status {
            db_status: row.get("db_status")?,
            last_update: row.get("last_update")?,
        })
    })?;

    if let Some(config) = config_iter.next() {
        match config {
            Ok(c) => Ok(c),
            Err(e) => Err(Box::new(e)),
        }
    } else {
        Ok(Status {
            db_status: "empty".to_string(),
            last_update: None,
        })
    }
}

pub fn update_status(conn: &Connection, status: &str) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(Sql::UpdateStatus.get().as_str(), params![status])?;
    Ok(())
}

pub fn select_rowcount(conn: &Connection) -> Result<RowCount, Box<dyn std::error::Error>> {
    let create_config = Sql::CreateConfig.get();
    let select_row_count = Sql::SelectRowCount.get();
    let mut stmt = match conn.prepare(&select_row_count) {
        Ok(s) => s,
        _ => {
            conn.execute_batch(&create_config)?;
            return Ok(RowCount {
                brands_rowcount: 0,
                models_rowcount: 0,
                years_rowcount: 0,
                vehicles_rowcount: 0,
            });
        }
    };

    let mut config_iter = stmt.query_map([], |row| {
        Ok(RowCount {
            brands_rowcount: row.get("brands_rowcount")?,
            models_rowcount: row.get("models_rowcount")?,
            years_rowcount: row.get("years_rowcount")?,
            vehicles_rowcount: row.get("vehicles_rowcount")?,
        })
    })?;

    if let Some(config) = config_iter.next() {
        Ok(config?)
    } else {
        Ok(RowCount {
            brands_rowcount: 0,
            models_rowcount: 0,
            years_rowcount: 0,
            vehicles_rowcount: 0,
        })
    }
}

pub fn select_count(conn: &Connection, entity: &str) -> Result<Count, Box<dyn std::error::Error>> {
    let select_count = (Sql::SelectCount {
        entity: entity.to_string(),
    })
    .get();
    let mut stmt = match conn.prepare(&select_count) {
        Ok(s) => s,
        _ => {
            return Ok(Count { count: 0 });
        }
    };

    let mut iter = stmt.query_map([], |row| Ok(Count { count: row.get(0)? }))?;

    if let Some(count) = iter.next() {
        match count {
            Ok(c) => Ok(c),
            Err(e) => Err(Box::new(e)),
        }
    } else {
        Ok(Count { count: 0 })
    }
}
pub fn update_rowcount(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let row_count = select_rowcount(conn)?;

    let entities = [
        ("brands", row_count.brands_rowcount),
        ("models", row_count.models_rowcount),
        ("years", row_count.years_rowcount),
        ("vehicles", row_count.vehicles_rowcount),
    ];

    for (name, current) in entities {
        let db_count = select_count(conn, name)?.count;

        if db_count > current {
            let update_row_count = (Sql::UpdateRowCount {
                entity: name.to_string(),
            })
            .get();
            conn.execute(&update_row_count, params![db_count])?;
        }
    }

    Ok(())
}
