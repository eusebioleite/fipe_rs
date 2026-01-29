use reqwest::Client;
use serde::Deserialize;
use rusqlite::{ params, Connection, Result };
use owo_colors::OwoColorize;
use std::process::exit;
use std::io::{ self, Write };

// Types
#[derive(Debug, Deserialize)]
struct Types {
    id: i32,
    description: String,
}

// References
#[derive(Debug, Deserialize)]
struct References {
    id: i32,
    month: String,
    year: String,
    fipe_id: String,
}

#[derive(Debug, Deserialize)]
struct ReferencesResponse {
    #[serde(rename = "Codigo")]
    codigo: i32,
    #[serde(rename = "Mes")]
    mes: String,
}

// Brands
#[derive(Debug, Deserialize)]
struct Brands {
    id: i32,
    description: String,
    fipe_id: String,
    r#type: String,
    reference: String,
}

// Models
struct Models {
    id: i32,
    description: String,
    fipe_id: String,
    brand: i32,
    r#type: i32,
    reference: i32,
}
#[derive(Debug, Deserialize)]
struct ModelsResponse {
    #[serde(rename = "Modelos")]
    model: Vec<FipeStruct>,
}

// Years
#[derive(Debug, Deserialize)]
struct FipeStruct {
    #[serde(rename = "Label")]
    label: String,
    #[serde(rename = "Value")]
    value: String,
}

// Utilities
#[derive(Debug, Deserialize)]
struct Config {
    db_status: String,
    last_update: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("fipe_rs.db")?;
    loop {
        clear_screen();
        println!("{}", label("app_title", &[]));

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
        println!("{}", label("db_status", &[&db_status]));

        let last_update_str = config.last_update.unwrap_or_else(|| "Never".to_string());
        println!("{}", label("last_update", &[&last_update_str]));
        println!("{}", label("1_recreate_db", &[]));
        println!("{}", label("2_load_references", &[]));
        println!("{}", label("3_load_brands", &[]));
        println!("{}", label("4_load_models", &[]));
        println!("{}", label("5_load_years", &[]));
        println!("{}", label("0_exit", &[]));
        print!("{}", label("user_input", &[]));
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice: usize = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("{}", label("invalid_input", &[]));
                println!("");
                continue;
            }
        };
        drop(input);

        match choice {
            1 => {
                match setup_db(&conn, true) {
                    Ok(_) => {
                        println!("{}", label("db_creation_ok", &[]));
                        update_status(&conn, "stable")?;
                    }
                    Err(e) => {
                        println!("{}", label("db_creation_err", &[&e.to_string()]));
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
            0 => {
                exit(0);
            }
            _ => {
                println!("{}", label("invalid_input", &[]));
                println!("");
                println!("");
            }
        }
    }
}

fn setup_db(conn: &Connection, recreate: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", label("db_open", &[]));

    if recreate {
        println!();

        // Years
        println!("{}", label("create_table", &["years"]));
        conn.execute_batch(&sql("create_years"))?;

        // Models
        println!("{}", label("create_table", &["models"]));
        conn.execute_batch(&sql("create_models"))?;

        // Brands
        println!("{}", label("create_table", &["brands"]));
        conn.execute_batch(&sql("create_brands"))?;

        // References
        println!("{}", label("create_table", &["references"]));
        conn.execute_batch(&sql("create_references"))?;

        // Types
        println!("{}", label("create_table", &["types"]));
        conn.execute_batch(&sql("create_types"))?;
        conn.execute(&sql("init_types"), ["Carros", "Motos", "Caminhões e Micro-Ônibus"])?;

        // Errors
        println!("{}", label("create_table", &["errors"]));
        conn.execute_batch(&sql("create_errors"))?;

        // Config
        println!("{}", label("create_table", &["config"]));
        conn.execute_batch(&sql("create_config"))?;

        Ok(())
    } else {
        Ok(())
    }
}

// Types
fn select_types(conn: &Connection) -> Result<Vec<Types>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(&sql("select_types"))?;
    let types_iter = stmt.query_map([], |row| {
        Ok(Types {
            id: row.get(0)?,
            description: row.get(1)?,
        })
    })?;

    let mut types = Vec::new();
    for t in types_iter {
        types.push(t?);
    }
    Ok(types)
}

// References
async fn load_references(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let response = match
        Client::new()
            .post("https://veiculos.fipe.org.br/api/veiculos/ConsultarTabelaDeReferencia")
            .header("Referer", "http://veiculos.fipe.org.br/")
            .header("Content-Type", "application/json")
            .send().await
    {
        Ok(r) => r,
        Err(e) => {
            println!("{}", label("response_error", &[&e.to_string()]));
            insert_error(
                conn,
                "References",
                "https://veiculos.fipe.org.br/api/veiculos/ConsultarTabelaDeReferencia",
                ""
            ).await?;
            exit(1);
        }
    };

    let references: Vec<ReferencesResponse> = response.json().await?;

    for reference in &references {
        let cod_str = reference.codigo.to_string();
        println!("{}", label("references_insert", &[&cod_str, &reference.mes]));
        conn.execute(
            &sql("insert_reference"),
            params![
                reference.mes.split('/').nth(0),
                reference.mes.split('/').nth(1),
                reference.codigo
            ]
        )?;
    }
    println!("{}", label("load_ok", &["References"]));
    Ok(())
}

fn select_references(conn: &Connection) -> Result<Vec<References>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(&sql("select_references"))?;
    let reference_iter = stmt.query_map([], |row| {
        Ok(References {
            id: row.get(0)?,
            month: row.get(1)?,
            year: row.get(2)?,
            fipe_id: row.get(3)?,
        })
    })?;

    let mut references = Vec::new();
    for reference in reference_iter {
        references.push(reference?);
    }
    Ok(references)
}

// Brands
async fn load_brands(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let types = select_types(conn)?;
    let references = select_references(conn)?;

    for t in &types {
        for reference in &references {
            let body =
                serde_json::json!({
                "codigoTipoVeiculo": t.id,
                "codigoTabelaReferencia": reference.fipe_id
            });

            let response = match
                Client::new()
                    .post("https://veiculos.fipe.org.br/api/veiculos/ConsultarMarcas")
                    .header("Referer", "http://veiculos.fipe.org.br/")
                    .header("Content-Type", "application/json")
                    .body(body.to_string())
                    .send().await
            {
                Ok(r) => r,
                Err(e) => {
                    println!("{}", label("response_error", &[&e.to_string()]));
                    insert_error(
                        conn,
                        "Brands",
                        "https://veiculos.fipe.org.br/api/veiculos/ConsultarMarcas",
                        &body.to_string()
                    ).await?;
                    exit(1);
                }
            };

            let brands: Vec<FipeStruct> = response.json().await?;
            for brand in brands {
                println!(
                    "{}",
                    label(
                        "brands_insert",
                        &[
                            &t.description,
                            &reference.month,
                            &reference.year,
                            &brand.label,
                            &brand.value,
                        ]
                    )
                );
                conn.execute(
                    &sql("insert_brands"),
                    params![brand.label, brand.value, t.id, reference.id]
                )?;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    }
    println!("{}", label("load_ok", &["Brands"]));
    Ok(())
}

fn select_brands(conn: &Connection) -> Result<Vec<Brands>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(&sql("select_brands"))?;
    let brand_iter = stmt.query_map([], |row| {
        Ok(Brands {
            id: row.get(0)?,
            description: row.get(1)?,
            fipe_id: row.get(2)?,
            r#type: row.get(3)?,
            reference: row.get(4)?,
        })
    })?;

    let mut brands = Vec::new();
    for brand in brand_iter {
        brands.push(brand?);
    }
    Ok(brands)
}

// Models
async fn load_models(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let brands = select_brands(conn)?;
    for brand in &brands {
        println!("{}", "Connecting to API...".bold().green());
        let body =
            serde_json::json!({
            "codigoTipoVeiculo": brand.r#type,
            "codigoTabelaReferencia": brand.reference,
            "codigoMarca": brand.fipe_id
        });

        let response = match
            Client::new()
                .post("https://veiculos.fipe.org.br/api/veiculos/ConsultarModelos")
                .header("Referer", "http://veiculos.fipe.org.br/")
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .send().await
        {
            Ok(r) => r,
            Err(e) => {
                println!("{}", label("response_error", &[&e.to_string()]));
                insert_error(
                    conn,
                    "Models",
                    "https://veiculos.fipe.org.br/api/veiculos/ConsultarModelos",
                    &body.to_string()
                ).await?;
                exit(1);
            }
        };

        let models: ModelsResponse = response.json().await?;
        for model in models.model {
            println!(
                "{}",
                label("models_insert", &[&brand.description, &model.value, &model.label])
            );
            conn.execute(&sql("insert_model"), params![model.label, model.value, brand.id])?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
    println!("{}", label("load_ok", &["Models"]));
    Ok(())
}

fn select_models(conn: &Connection) -> Result<Vec<Models>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(&sql("select_models"))?;
    let model_iter = stmt.query_map([], |row| {
        Ok(Models {
            id: row.get(0)?,
            description: row.get(1)?,
            fipe_id: row.get(2)?,
            brand: row.get(3)?,
            r#type: row.get(4)?,
            reference: row.get(5)?,
        })
    })?;

    let mut models = Vec::new();
    for model in model_iter {
        models.push(model?);
    }
    Ok(models)
}

// Years
async fn load_years(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let models = select_models(conn)?;
    for model in &models {
        println!("{}", "Connecting to API...".bold().green());
        let body =
            serde_json::json!({
            "codigoTipoVeiculo": model.r#type,
            "codigoTabelaReferencia": model.reference,
            "codigoMarca": model.brand,
            "codigoModelo": model.fipe_id
        });

        let response = match
            Client::new()
                .post("https://veiculos.fipe.org.br/api/veiculos/ConsultarAnoModelo")
                .header("Referer", "http://veiculos.fipe.org.br/")
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .send().await
        {
            Ok(r) => r,
            Err(e) => {
                println!("{}", label("response_error", &[&e.to_string()]));
                insert_error(
                    conn,
                    "Years",
                    "https://veiculos.fipe.org.br/api/veiculos/ConsultarAnosModelo",
                    &body.to_string()
                ).await?;
                exit(1);
            }
        };

        let years: Vec<FipeStruct> = response.json().await?;
        for year in years {
            println!("{}", label("years_insert", &[&model.description, &year.value, &year.label]));
            conn.execute(&sql("insert_year"), params![year.label, year.value, model.id])?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
    println!("{}", label("load_ok", &["Years"]));
    Ok(())
}
async fn insert_error(
    conn: &Connection,
    entity: &str,
    url: &str,
    body: &str
) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(&sql("insert_error"), params![entity, url, body])?;
    Ok(())
}

async fn select_status(conn: &Connection) -> Result<Config, Box<dyn std::error::Error>> {
    let mut stmt = match conn.prepare(&sql("select_config")) {
        Ok(s) => s,
        _ => {
            conn.execute_batch(&sql("create_config_table"))?;
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

fn update_status(conn: &Connection, status: &str) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(&sql("update_config"), params![status])?;
    Ok(())
}
fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn press_key_continue() {
    println!("");
    println!("{}", label("press_key_continue", &[]));
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

fn label(query: &str, params: &[&str]) -> String {
    let s = match query {
        // main
        "app_title" => format!("{}{}", "FIPE".bold().bright_cyan(), "_rs".bold().red()),
        "db_status" => format!("{}: {}", "DB Status".bold().yellow(), params[0]),
        "last_update" =>
            format!(
                "{}: {}",
                "Last Update".bright_black().italic().dimmed().underline(),
                params[0].bold().bright_black().italic().underline()
            ),
        "1_recreate_db" =>
            format!("{} - {}", "1".bold().bright_green(), "Recreate Database".bold()),
        "2_load_references" =>
            format!("{} - {}", "2".bold().bright_green(), "Load References".bold()),
        "3_load_brands" => format!("{} - {}", "3".bold().bright_green(), "Load Brands".bold()),
        "4_load_models" => format!("{} - {}", "4".bold().bright_green(), "Load Models".bold()),
        "5_load_years" => format!("{} - {}", "5".bold().bright_green(), "Load Years".bold()),
        "0_exit" => format!("{} - {}", "0".bold().bright_green(), "Exit".bold()),
        "user_input" => format!("{}", "> ".bold().bright_cyan()),
        "invalid_input" => format!("{}", "Invalid input.".bold().bright_red()),
        "db_creation_ok" =>
            format!("{}", "Succesfully recreated the database.".bold().bright_green()),
        "db_creation_err" =>
            format!("{} : {}", "[ERROR]".bold().bright_red(), params[0].italic().bright_black()),

        // setup_db
        "db_open" => format!("{}", "Opening database...".bold().green()),
        "create_table" => format!("CREATE TABLE: {} ...", params[0].to_string().bold().cyan()),

        // requests
        "response_error" =>
            format!("{} : {}", "[ERROR]".bold().red(), params[0].to_string().italic().white()),
        "load_ok" => format!("{}", format!("{} successfully loaded.", params[0]).bold().green()),
        "references_insert" =>
            format!("{}: {} - {}", "INSERT".bright_blue().bold(), params[0], params[1]),
        "brands_insert" =>
            format!(
                "{} | {} | {}: {} - {}",
                params[0].to_string().cyan().bold(),
                String::from(format!("{}/{}", params[1], params[2])).yellow().bold(),
                "INSERT".to_string().bright_blue().bold(),
                params[3],
                params[4]
            ),
        "models_insert" =>
            format!(
                "{} | {}: {} - {}",
                params[0].to_string().cyan().bold(),
                "INSERT".to_string().bright_blue().bold(),
                params[1],
                params[2]
            ),
        "years_insert" =>
            format!(
                "{} | {}: {} - {}",
                params[0].to_string().cyan().bold(),
                "INSERT".to_string().bright_blue().bold(),
                params[1],
                params[2]
            ),
        "press_key_continue" => format!("{}", "Press ENTER to continue...".bold().bright_black()),
        _ => "Unknown".to_string(),
    };
    s.to_string()
}

fn sql(query: &str) -> String {
    let s = match query {
        // Table Creation (Setup)
        "create_years" =>
            r#"
            DROP TABLE IF EXISTS years;
            CREATE TABLE years(
                id integer PRIMARY KEY,
                description text,
                fipe_id text,
                model integer,
                foreign key(model) references models(id)
            )"#,
        "create_models" =>
            r#"
            DROP TABLE IF EXISTS models;
            CREATE TABLE models(
                id integer PRIMARY KEY,
                description text,
                fipe_id text,
                brand integer,
                foreign key(brand) references brands(id)
            )"#,
        "create_brands" =>
            r#"
            DROP TABLE IF EXISTS brands;
            CREATE TABLE brands(
                id integer PRIMARY KEY,
                description text,
                fipe_id text,
                type integer,
                reference integer,
                foreign key(type) references types(id),
                foreign key(reference) references "references"(id)
            )"#,
        "create_references" =>
            r#"
            DROP TABLE IF EXISTS "references";
            CREATE TABLE "references"(
                id integer PRIMARY KEY,
                month text,
                year text,
                fipe_id text
            )"#,
        "create_types" =>
            r#"
            DROP TABLE IF EXISTS types;
            CREATE TABLE types(
                id integer PRIMARY KEY,
                description text
            )"#,
        "init_types" => r#"INSERT INTO types(description) VALUES (?1), (?2), (?3)"#,
        "create_errors" =>
            r#"
            DROP TABLE IF EXISTS errors;
            CREATE TABLE errors(
                id int,
                entity text,
                url text,
                body text
            );"#,
        "create_config" =>
            r#"
            DROP TABLE IF EXISTS config;
            CREATE TABLE config(
                db_status text, -- empty, stable, compromised
                last_update date
            );

            INSERT INTO config VALUES ('empty', datetime('now'));

            CREATE TRIGGER config_single_row
            BEFORE INSERT ON config
            WHEN (SELECT COUNT(*) FROM config) >= 1
            BEGIN
                SELECT RAISE(ABORT, 'config can only have a single row.');
            END;"#,

        // Selects & Inserts
        "select_types" => "SELECT id, description FROM types",

        "insert_reference" =>
            "INSERT INTO \"references\" (month, year, fipe_id) VALUES (?1, ?2, ?3)",
        "select_references" => "SELECT id, month, year, fipe_id FROM \"references\"",

        "insert_brand" =>
            "INSERT INTO brands (description, fipe_id, type, reference) VALUES (?1, ?2, ?3, ?4)",
        "select_brands" => "SELECT id, description, fipe_id, type, reference FROM brands",

        "insert_model" => "INSERT INTO models (description, fipe_id, brand) VALUES (?1, ?2, ?3)",
        "select_models" =>
            r#"
            SELECT models.id, models.description, models.fipe_id, models.brand, brands.type, brands.reference
            FROM models LEFT JOIN brands ON models.brand = brands.id
        "#,

        "insert_year" => "INSERT INTO years (description, fipe_id, model) VALUES (?1, ?2, ?3)",

        "insert_error" => "INSERT INTO errors (entity, url, body) VALUES (?1, ?2, ?3)",

        "select_config" => "SELECT db_status, last_update FROM config",
        "update_config" => "UPDATE config SET db_status = ?1, last_update = datetime('now')",

        _ => "",
    };
    s.to_string()
}
