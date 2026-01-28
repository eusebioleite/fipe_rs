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
    Codigo: i32,
    Mes: String,
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
#[derive(Debug, Deserialize)]
struct BrandsResponse {
    Label: String,
    Value: String,
}
#[derive(Debug, Deserialize)]

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
    Modelos: Vec<ModelsResponseModels>,

    #[allow(dead_code)]
    #[serde(skip_deserializing)]
    Anos: Vec<ModelsResponseYears>,
}
#[derive(Debug, Deserialize)]
struct ModelsResponseModels {
    Label: String,
    Value: i32,
}
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ModelsResponseYears {
    label: i32,
    value: String,
}

// Years
#[derive(Debug, Deserialize)]
struct YearsResponse {
    Label: String,
    Value: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "FIPE Helper".bold().cyan());
    println!("{}Recreate database.", "1 - ".to_string().bold().green());
    println!("{}Load references.", "2 - ".to_string().bold().green());
    println!("{}Load brands.", "3 - ".to_string().bold().green());
    println!("{}Load models.", "4 - ".to_string().bold().green());
    println!("{}Load years.", "5 - ".to_string().bold().green());
    println!("{}Exit.", "0 - ".to_string().bold().green());
    print!("{}", String::from("> ").bold().cyan());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice: usize = input.trim().parse()?;
    drop(input);

    match choice {
        1 => {
            match setup_db(true) {
                Ok(_) => println!("{}", "Succesfully recreated the database.".bold().green()),
                Err(e) =>
                    println!("{} : {}", "[ERROR]".bold().red(), e.to_string().italic().white()),
            }
        }
        2 => {
            let conn = setup_db(false)?;
            load_references(&conn).await?;
        }
        3 => {
            let conn = setup_db(false)?;
            load_brands(&conn).await?;
        }
        4 => {
            let conn = setup_db(false)?;
            load_models(&conn).await?;
        }
        5 => {
            let conn = setup_db(false)?;
            load_years(&conn).await?;
        }
        0 => {
            exit(0);
        }
        _ => {
            println!("{}", "Invalid option.".bold().red());
            exit(1);
        }
    }

    Ok(())
}

fn setup_db(recreate: bool) -> Result<Connection, Box<dyn std::error::Error>> {
    println!("{}", "Opening database...".bold().green());
    let conn = Connection::open("rustfipe.db")?;

    if recreate {
        println!("{}", "Recreating database...".bold().green());

        // Years
        println!("{} table: {}...", "DROP".red().bold(), "years".to_string().bold().cyan());
        conn.execute("DROP TABLE IF EXISTS years", [])?;
        println!("{} table: {}...", "CREATE".yellow().bold(), "years".to_string().bold().cyan());
        conn.execute(
            r#"
                CREATE TABLE years(
                    id integer PRIMARY KEY,
                    description text,
                    fipe_id text,
                    model integer,
                    foreign key(model) references models(id)
                )
            "#,
            []
        )?;

        // Models
        println!("{} table: {}...", "DROP".red().bold(), "models".to_string().bold().cyan());
        conn.execute("DROP TABLE IF EXISTS models", [])?;
        println!("{} table: {}...", "CREATE".yellow().bold(), "models".to_string().bold().cyan());
        conn.execute(
            r#"
                CREATE TABLE models(
                    id integer PRIMARY KEY,
                    description text,
                    fipe_id text,
                    brand integer,
                    foreign key(brand) references brands(id)
                )
            "#,
            []
        )?;

        // Brands
        println!("{} table: {}...", "DROP".red().bold(), "brands".to_string().bold().cyan());
        conn.execute("DROP TABLE IF EXISTS brands", [])?;
        println!("{} table: {}...", "CREATE".yellow().bold(), "brands".to_string().bold().cyan());
        conn.execute(
            r#"
                CREATE TABLE brands(
                    id integer PRIMARY KEY,
                    description text,
                    fipe_id text,
                    type integer,
                    reference integer,
                    foreign key(type) references types(id),
                    foreign key(reference) references "references"(id)
                )
            "#,
            []
        )?;

        // References
        println!("{} table: {}...", "DROP".red().bold(), "references".to_string().bold().cyan());
        conn.execute("DROP TABLE IF EXISTS \"references\"", [])?;
        println!(
            "{} table: {}...",
            "CREATE".yellow().bold(),
            "references".to_string().bold().cyan()
        );
        conn.execute(
            r#"
            CREATE TABLE "references"(
                id integer PRIMARY KEY,
                month text,
                year text,
                fipe_id text
            )
        "#,
            []
        )?;

        // Types
        println!("{} table: {}...", "DROP".red().bold(), "types".to_string().bold().cyan());
        conn.execute("DROP TABLE IF EXISTS types", [])?;
        println!("{} table: {}...", "CREATE".yellow().bold(), "types".to_string().bold().cyan());
        conn.execute(
            r#"
            CREATE TABLE types(
                id integer PRIMARY KEY,
                description text
            )
        "#,
            []
        )?;
        println!("INSERT default values into: {}...", "types".to_string().bold().cyan());
        conn.execute(r#"INSERT INTO types(description) VALUES (?1), (?2), (?3)"#, [
            "Carros",
            "Motos",
            "Caminhões e Micro-Ônibus",
        ])?;
    }

    Ok(conn)
}

// Types
fn select_types() -> Result<Vec<Types>, Box<dyn std::error::Error>> {
    let conn = Connection::open("rustfipe.db")?;
    let mut stmt = conn.prepare("SELECT id, description FROM types")?;
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
    println!("{}", "Connecting to API...".bold().green());
    let response = Client::new()
        .post("https://veiculos.fipe.org.br/api/veiculos/ConsultarTabelaDeReferencia")
        .header("Referer", "http://veiculos.fipe.org.br/")
        .header("Content-Type", "application/json")
        .send().await?;

    let references: Vec<ReferencesResponse> = response.json().await?;

    for reference in &references {
        println!("{}: {} - {}", "INSERT".blue().bold(), reference.Codigo, reference.Mes);
        conn.execute(
            "INSERT INTO \"references\" (month, year, fipe_id) VALUES (?1, ?2, ?3)",
            params![
                reference.Mes.split('/').nth(0),
                reference.Mes.split('/').nth(1),
                reference.Codigo
            ]
        )?;
    }
    println!("{}", "References successfully loaded.".bold().green());
    Ok(())
}

fn select_references() -> Result<Vec<References>, Box<dyn std::error::Error>> {
    let conn = Connection::open("rustfipe.db")?;
    let mut stmt = conn.prepare("SELECT id, month, year, fipe_id FROM \"references\"")?;
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
    let types = select_types()?;
    let references = select_references()?;

    for t in &types {
        for reference in &references {
            println!("{}", "Connecting to API...".bold().green());
            let body =
                serde_json::json!({
                "codigoTipoVeiculo": t.id,
                "codigoTabelaReferencia": reference.fipe_id
            });

            let response = Client::new()
                .post("https://veiculos.fipe.org.br/api/veiculos/ConsultarMarcas")
                .header("Referer", "http://veiculos.fipe.org.br/")
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .send().await?;

            let brands: Vec<BrandsResponse> = response.json().await?;
            for brand in brands {
                println!(
                    "{} | {} | {}: {} - {}",
                    t.description.to_string().cyan().bold(),
                    String::from(format!("{}/{}", reference.month, reference.year)).yellow().bold(),
                    "INSERT".to_string().blue().bold(),
                    brand.Value,
                    brand.Label
                );
                conn.execute(
                    r"INSERT INTO brands (description, fipe_id, type, reference) VALUES (?1, ?2, ?3, ?4)",
                    params![brand.Label, brand.Value, t.id, reference.id]
                )?;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    }
    println!("{}", "Brands successfully loaded.".bold().green());
    Ok(())
}

fn select_brands() -> Result<Vec<Brands>, Box<dyn std::error::Error>> {
    let conn = Connection::open("rustfipe.db")?;
    let mut stmt = conn.prepare("SELECT id, description, fipe_id, type, reference FROM brands")?;
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
    let brands = select_brands()?;
    for brand in &brands {
        println!("{}", "Connecting to API...".bold().green());
        let body =
            serde_json::json!({
            "codigoTipoVeiculo": brand.r#type,
            "codigoTabelaReferencia": brand.reference,
            "codigoMarca": brand.fipe_id
        });

        let response = Client::new()
            .post("https://veiculos.fipe.org.br/api/veiculos/ConsultarModelos")
            .header("Referer", "http://veiculos.fipe.org.br/")
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send().await?;

        let models: ModelsResponse = response.json().await?;
        for model in models.Modelos {
            println!(
                "{} | {}: {} - {}",
                brand.description.to_string().cyan().bold(),
                "INSERT".to_string().blue().bold(),
                model.Value,
                model.Label
            );
            conn.execute(
                r"INSERT INTO models (description, fipe_id, brand) VALUES (?1, ?2, ?3)",
                params![model.Label, model.Value, brand.id]
            )?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
    println!("{}", "Models successfully loaded.".bold().green());
    Ok(())
}

fn select_models() -> Result<Vec<Models>, Box<dyn std::error::Error>> {
    let conn = Connection::open("rustfipe.db")?;
    let mut stmt = conn.prepare(
        r#"
        SELECT id, description, fipe_id, brand, type, reference
        FROM models LEFT JOIN brands ON models.brand = brands.id
    "#
    )?;
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
    let models = select_models()?;
    for model in &models {
        println!("{}", "Connecting to API...".bold().green());
        let body =
            serde_json::json!({
            "codigoTipoVeiculo": model.r#type,
            "codigoTabelaReferencia": model.reference,
            "codigoMarca": model.brand,
            "codigoModelo": model.fipe_id
        });

        let response = Client::new()
            .post("https://veiculos.fipe.org.br/api/veiculos/ConsultarAnoModelo")
            .header("Referer", "http://veiculos.fipe.org.br/")
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send().await?;

        let years: Vec<YearsResponse> = response.json().await?;
        for year in years {
            println!(
                "{} | {}: {} - {}",
                model.description.to_string().cyan().bold(),
                "INSERT".to_string().blue().bold(),
                year.Value,
                year.Label
            );
            conn.execute(
                r"INSERT INTO years (description, fipe_id, model) VALUES (?1, ?2, ?3)",
                params![year.Label, year.Value, model.id]
            )?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
    println!("{}", "Years successfully loaded.".bold().green());
    Ok(())
}
