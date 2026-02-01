use owo_colors::OwoColorize;
use std::fmt;

pub enum Sql {
    // setup
    CreateYears,
    CreateModels,
    CreateBrands,
    CreateReferences,
    CreateTypes,
    InitTypes,
    CreateErrors,
    CreateConfig,

    // selects
    SelectTypes,
    SelectReferences,
    SelectBrands,
    SelectModels,
    SelectConfig,

    // inserts / updates
    InsertReference,
    InsertBrand,
    InsertModel,
    InsertYear,
    InsertError,
    UpdateConfig,
}

pub enum Label<'a> {
    // main
    MenuOptions {
        db_status: &'a str,
        last_update: &'a str,
    },
    InvalidInput,
    DbCreationOk,
    DbCreationErr {
        message: &'a str,
    },

    // setup_db
    CreateTable {
        table_name: &'a str,
    },

    // requests
    ResponseError {
        message: &'a str,
    },
    LoadOk {
        entity: &'a str,
    },
    InsertReference {
        codigo: &'a str,
        mes: &'a str,
    },
    InsertBrand {
        tipo: &'a str,
        referencia: &'a str,
        marca: &'a str,
        codigo: &'a str,
    },
    InsertModel {
        tipo: &'a str,
        referencia: &'a str,
        marca: &'a str,
        modelo: &'a str,
        codigo: &'a str,
    },
    InsertYear {
        tipo: &'a str,
        referencia: &'a str,
        marca: &'a str,
        modelo: &'a str,
        ano: &'a str,
        codigo: &'a str,
    },
    PressKeyContinue,
}

impl<'a> fmt::Display for Label<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Label::MenuOptions {
                db_status,
                last_update,
            } => write!(
                f,
                "{}\n{} {}\n{} {}\n{} - {}\n{} - {}\n{} - {}\n{} - {}\n{} - {}\n{} - {}\n",
                "FIPE_rs".bold().bright_cyan(),
                "DB Status:".bold().yellow(),
                db_status.bold(),
                "Last Update:".bold().black().dimmed(),
                last_update.italic().black().dimmed(),
                "1".bold().bright_green(),
                "Recreate Database",
                "2".bold().bright_green(),
                "Load References",
                "3".bold().bright_green(),
                "Load Brands",
                "4".bold().bright_green(),
                "Load Models",
                "5".bold().bright_green(),
                "Load Years",
                "0".bold().bright_green(),
                "Exit"
            ),
            Label::InvalidInput => write!(
                f,
                "{}: {}",
                "[ERROR]".bold().bright_red(),
                "Invalid input.".italic().black().dimmed()
            ),
            Label::DbCreationOk => write!(
                f,
                "{}: {}",
                "[SUCCESS]".bold().bright_green(),
                "Succesfully recreated the database.".bold()
            ),
            Label::DbCreationErr { message } | Label::ResponseError { message } => write!(
                f,
                "{}: {}",
                "[ERROR]".bold().bright_red(),
                message.italic().black().dimmed()
            ),
            Label::CreateTable { table_name } => write!(
                f,
                "{}: {}",
                "[SUCCESS]".bold().bright_green(),
                format!("Table {} created.", table_name.blue()).bold()
            ),
            Label::LoadOk { entity } => write!(
                f,
                "{}: {}",
                "[SUCCESS]".bold().bright_green(),
                format!(" {} successfully loaded.", entity.blue()).bold()
            ),
            Label::InsertReference { codigo, mes } => write!(
                f,
                "{}: {} - {}",
                "[SUCCESS]".bold().bright_green(),
                codigo,
                mes
            ),
            Label::InsertBrand {
                tipo,
                referencia,
                marca,
                codigo,
            } => write!(
                f,
                "{}: | {} | {} {} - {}",
                tipo.bold().blue(),
                referencia.bold().yellow(),
                "[SUCCESS]".bold().bright_green(),
                marca,
                codigo
            ),
            Label::InsertModel {
                tipo,
                referencia,
                marca,
                modelo,
                codigo,
            } => write!(
                f,
                "{}: | {} | {} | {} {} - {}",
                tipo.bold().blue(),
                referencia.bold().yellow(),
                marca.bold().red(),
                "[SUCCESS]".bold().bright_green(),
                modelo,
                codigo
            ),
            Label::InsertYear {
                tipo,
                referencia,
                marca,
                modelo,
                ano,
                codigo,
            } => write!(
                f,
                "{}: | {} | {} | {} | {} {} - {}",
                tipo.bold().blue(),
                referencia.bold().yellow(),
                marca.bold().red(),
                modelo.bold().magenta(),
                "[SUCCESS]".bold().bright_green(),
                ano,
                codigo
            ),
            Label::PressKeyContinue => write!(
                f,
                "{}",
                "Press any key to continue...".italic().black().dimmed()
            ),
        }
    }
}

impl<'a> Label<'a> {
    pub fn log(&self) {
        println!("{}", self);
    }
}
impl Sql {
    pub fn as_str(&self) -> &'static str {
        match self {
            // setup
            Sql::CreateYears => {
                r#"
                DROP TABLE IF EXISTS years;
                CREATE TABLE years(
                    id integer PRIMARY KEY,
                    description text,
                    fipe text unique,
                    model_id integer,
                    foreign key(model_id) references models(id)
                )
            "#
            }

            Sql::CreateModels => {
                r#"
                DROP TABLE IF EXISTS models;
                CREATE TABLE models(
                    id integer PRIMARY KEY,
                    description text,
                    fipe text unique,
                    brand_id integer,
                    foreign key(brand_id) references brands(id)
                )
            "#
            }

            Sql::CreateBrands => {
                r#"
                DROP TABLE IF EXISTS brands;
                CREATE TABLE brands(
                    id integer PRIMARY KEY,
                    description text,
                    fipe text unique,
                    type_id integer,
                    ref_id integer,
                    foreign key(type_id) references types(id),
                    foreign key(ref_id) references "references"(id)
                )
            "#
            }

            Sql::CreateReferences => {
                r#"
                DROP TABLE IF EXISTS "references";
                CREATE TABLE "references"(
                    id integer PRIMARY KEY,
                    month text,
                    year text,
                    fipe text unique
                )
            "#
            }

            Sql::CreateTypes => {
                r#"
                DROP TABLE IF EXISTS types;
                CREATE TABLE types(
                    id integer PRIMARY KEY,
                    description text
                )
            "#
            }

            Sql::InitTypes => "INSERT INTO types(description) VALUES (?1), (?2), (?3)",

            Sql::CreateErrors => {
                r#"
                DROP TABLE IF EXISTS errors;
                CREATE TABLE errors(
                    id int,
                    entity text,
                    url text,
                    body text
                );
            "#
            }

            Sql::CreateConfig => {
                r#"
                DROP TABLE IF EXISTS config;
                CREATE TABLE config(
                    db_status text,
                    last_update date
                );

                INSERT INTO config VALUES ('empty', datetime('now', 'localtime'));

                CREATE TRIGGER config_single_row
                BEFORE INSERT ON config
                WHEN (SELECT COUNT(*) FROM config) >= 1
                BEGIN
                    SELECT RAISE(ABORT, 'config can only have a single row.');
                END;
            "#
            }

            // selects
            Sql::SelectTypes => "SELECT id, description FROM types",

            Sql::SelectReferences => "SELECT id, month, year, fipe FROM \"references\"",

            Sql::SelectBrands => {
                r#"
                SELECT
                    b.id AS id,
                    b.description,
                    b.fipe,
                    r.fipe ref_id,
                    r."month" || '/' || r."year" ref_description,
                    b.type_id,
                    t.description type_description
                FROM brands b
                LEFT JOIN "references" r ON b.ref_id = r.id
                LEFT JOIN types t ON b.type_id = t.id;
            "#
            }

            Sql::SelectModels => {
                r#"
                SELECT
                    m.id,
                    m.description,
                    m.fipe,
                    b.fipe brand_id,
                    b.description brand_description,
                    r.fipe AS ref_id,
                    r."month" || '/' || r."year" ref_description,
                    b.type_id type_id,
                    t.description type_description
                FROM models m
                LEFT JOIN brands b ON m.brand_id = b.id
                LEFT JOIN "references" r ON b.ref_id = r.id
                LEFT JOIN types t ON b.type_id = t.id
            "#
            }

            Sql::SelectConfig => "SELECT db_status, last_update FROM config",

            // inserts / updates
            Sql::InsertReference => {
                "INSERT INTO \"references\" (month, year, fipe) VALUES (?1, ?2, ?3)"
            }

            Sql::InsertBrand => {
                "INSERT INTO brands (description, fipe, type_id, ref_id) VALUES (?1, ?2, ?3, ?4)"
            }

            Sql::InsertModel => {
                "INSERT INTO models (description, fipe, brand_id) VALUES (?1, ?2, ?3)"
            }

            Sql::InsertYear => {
                "INSERT INTO years (description, fipe, model_id) VALUES (?1, ?2, ?3)"
            }

            Sql::InsertError => "INSERT INTO errors (entity, url, body) VALUES (?1, ?2, ?3)",

            Sql::UpdateConfig => {
                "UPDATE config SET db_status = ?1, last_update = datetime('now', 'localtime')"
            }
        }
    }
}
