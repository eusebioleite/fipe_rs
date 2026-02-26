use crate::label::Label;
use crate::schema::{Count, RowCount, Status};
use crate::sql::Sql;
use crate::utils::{clear_screen, progress_bar};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rusqlite::{params, Connection, Result};

pub fn check_db(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    clear_screen();
    let m = MultiProgress::new();
    let style = ProgressStyle::with_template(
        "{msg}{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
    )
    .unwrap()
    .progress_chars("=> ");
    let counts = match select_rowcount(conn) {
        Ok(counts) => counts,
        Err(err) => {
            panic!("lmao {}", err)
        }
    };

    // Definição das barras
    let pb_years = m.add(ProgressBar::new(
        counts.years_rowcount.try_into().unwrap_or(0),
    ));
    pb_years.set_style(style.clone());
    pb_years.set_message("References");

    let pb_brands = m.add(ProgressBar::new(
        counts.brands_rowcount.try_into().unwrap_or(0),
    ));
    pb_brands.set_style(style.clone());
    pb_brands.set_message("Brands");

    let pb_models = m.add(ProgressBar::new(
        counts.models_rowcount.try_into().unwrap_or(0),
    ));
    pb_models.set_style(style.clone());
    pb_models.set_message("Models");

    // Persistência: Impede que as barras sejam limpas ao finalizar
    pb_years.finish_with_message("Years");
    pb_brands.finish_with_message("Brands");
    pb_models.finish_with_message("Models");

    // Contagem atual
    let years_count: u64 = match select_count(conn, "years") {
        Ok(count) => count.count.try_into().unwrap_or(0),
        Err(_) => 0,
    };
    let brands_count: u64 = match select_count(conn, "brands") {
        Ok(count) => count.count.try_into().unwrap_or(0),
        Err(_) => 0,
    };
    let models_count: u64 = match select_count(conn, "models") {
        Ok(count) => count.count.try_into().unwrap_or(0),
        Err(_) => 0,
    };

    // Simulação de atualização baseada nos seus counts
    pb_years.set_position(years_count);
    pb_brands.set_position(brands_count);
    pb_models.set_position(models_count);
    Ok(())
}
pub fn setup_db(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
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

/*
stable
empty
updated
outdated
*/
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
