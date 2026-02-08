use inquire::Select;
use owo_colors::OwoColorize;
use std::fmt;

pub enum Sql {
    // setup
    DropTables,
    CreateYears,
    CreateModels,
    CreateBrands,
    CreateReferences,
    CreateTypes,
    CreateFuels,
    InitFuels,
    InitTypes,
    CreateIndexes,
    CreateConfig,

    // selects
    SelectTypes,
    SelectReferences,
    SelectBrands,
    SelectModels,
    SelectModelsReplicate,
    SelectConfig,

    // inserts / updates
    InsertReference,
    InsertBrand,
    InsertModel,
    InsertYear,
    UpdateConfig,
}

pub enum Label<'a> {
    // main
    Header {
        db_status: &'a str,
        last_update: &'a str,
    },

    // setup_db
    CreateTable {
        table_name: &'a str,
    },
    CreateIndexes,
    // requests
    ResponseError {
        message: &'a str,
    },
    ApiConnectionError {
        message: &'a str,
    },
    ApiBlock {
        code: &'a str,
    },
    LoadOk {
        entity: &'a str,
    },
    UniqueConstraint {
        fipe: &'a str,
    },
    TableNotExist,
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

pub enum MainMenu {
    Loads,
    Maintenance,
    Exit,
}
pub enum MaintMenu {
    RecreateDatabase,
    CheckUpdates,
    Back,
}

pub enum LoadMenu {
    LoadRefs,
    LoadBrands,
    LoadModels,
    LoadYears,
    Back,
}

impl<'a> fmt::Display for Label<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Label::Header {
                db_status,
                last_update,
            } => write!(
                f,
                "{}\n{} {}\n{} {}\n",
                "FIPE_rs".bold().bright_cyan(),
                "DB Status:".bold().yellow(),
                db_status.bold(),
                "Last Update:".bold().black().dimmed(),
                last_update.italic().black().dimmed()
            ),
            Label::ResponseError { message } | Label::ApiConnectionError { message } => write!(
                f,
                "{}: {}",
                "[ERROR]".bold().bright_red(),
                message.italic().black().dimmed()
            ),
            Label::ApiBlock { code } => write!(
                f,
                "{}: {} {}",
                "[ERROR]".bold().bright_red(),
                code.italic().black().dimmed(),
                "Too many requests - API blocking, waiting 60 seconds..."
                    .italic()
                    .black()
                    .dimmed()
            ),
            Label::CreateTable { table_name } => write!(
                f,
                "{}: {}",
                "[SUCCESS]".bold().bright_green(),
                format!("Table {} created.", table_name.blue()).bold()
            ),
            Label::CreateIndexes => write!(
                f,
                "{}: {}",
                "[SUCCESS]".bold().bright_green(),
                "Created indexes.".bold()
            ),
            Label::LoadOk { entity } => write!(
                f,
                "{}: {}",
                "[SUCCESS]".bold().bright_green(),
                format!(" {} successfully loaded.", entity.blue()).bold()
            ),
            Label::UniqueConstraint { fipe } => {
                write!(
                    f,
                    "{}: {} {}",
                    "[WARN]".bold().yellow(),
                    fipe.italic().black().dimmed(),
                    "Already exists.".italic().black().dimmed()
                )
            }
            Label::InsertReference { codigo, mes } => write!(
                f,
                "{}: {} - {}",
                "[SUCCESS]".bold().bright_green(),
                codigo,
                mes
            ),
            Label::TableNotExist => write!(
                f,
                "{}: {}",
                "[ERROR]".bold().bright_red(),
                "A table does not exist! Please recreate the database first."
                    .italic()
                    .black()
                    .dimmed()
            ),
            Label::InsertBrand {
                tipo,
                referencia,
                marca,
                codigo,
            } => write!(
                f,
                "{} | {} | {}: {} - {}",
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
                "{} | {} | {} | {}: {} - {}",
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
                "{} | {} | {} | {} | {}: {} - {}",
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

impl fmt::Display for MainMenu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MainMenu::Loads => write!(f, "📥 Loads"),
            MainMenu::Maintenance => write!(f, "🛠️  Maintenance"),
            MainMenu::Exit => write!(f, "🔌 Exit"),
        }
    }
}

impl fmt::Display for LoadMenu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoadMenu::LoadRefs => write!(f, "Load References"),
            LoadMenu::LoadBrands => write!(f, "Load Brands"),
            LoadMenu::LoadModels => write!(f, "Load Models"),
            LoadMenu::LoadYears => write!(f, "Load Years"),
            LoadMenu::Back => write!(f, "Back"),
        }
    }
}

impl fmt::Display for MaintMenu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaintMenu::RecreateDatabase => write!(f, "Recreate Database"),
            MaintMenu::CheckUpdates => write!(f, "Check for Updates"),
            MaintMenu::Back => write!(f, "Back"),
        }
    }
}

impl Sql {
    pub fn as_str(&self) -> &'static str {
        match self {
            // setup
            Sql::DropTables => {
                r#"
                DROP TABLE IF EXISTS config;
                DROP TABLE IF EXISTS years;
                DROP TABLE IF EXISTS models;
                DROP TABLE IF EXISTS brands;
                DROP TABLE IF EXISTS "references";
                DROP TABLE IF EXISTS fuels;
                DROP TABLE IF EXISTS types;
            "#
            }

            Sql::CreateYears => {
                r#"
                CREATE TABLE years(
                    id integer PRIMARY KEY,
                    description text,
                    fipe text,
                    model_id integer,
                    fuel_id integer,
                    foreign key(model_id) references models(id),
                    foreign key(fuel_id) references fuels(id),
                    unique(fipe, model_id)
                )
            "#
            }

            Sql::CreateModels => {
                r#"
                DROP TABLE IF EXISTS models;
                CREATE TABLE models(
                    id integer PRIMARY KEY,
                    description text,
                    fipe text,
                    brand_id integer,
                    foreign key(brand_id) references brands(id),
                    unique(fipe, brand_id)
                )
            "#
            }

            Sql::CreateBrands => {
                r#"
                CREATE TABLE brands(
                    id integer PRIMARY KEY,
                    description text,
                    fipe text,
                    type_id integer,
                    ref_id integer,
                    foreign key(type_id) references types(id),
                    foreign key(ref_id) references "references"(id),
                    unique(fipe, ref_id)
                )
            "#
            }

            Sql::CreateReferences => {
                r#"
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
                CREATE TABLE types(
                    id integer PRIMARY KEY,
                    description text
                )
            "#
            }

            Sql::CreateFuels => {
                r#"
                CREATE TABLE fuels(
                    id integer PRIMARY KEY,
                    description text
                )
            "#
            }

            Sql::InitTypes => "INSERT INTO types(description) VALUES (?1), (?2), (?3)",

            Sql::InitFuels =>
                "INSERT INTO fuels(description) VALUES (?1), (?2), (?3), (?4), (?5), (?6), (?7), (?8)",

            Sql::CreateIndexes => {
                r#"
                CREATE INDEX idx_references_id ON "references" (id);
                CREATE INDEX idx_types_id ON types (id);
                CREATE INDEX idx_fuels_id ON fuels (id);
                CREATE INDEX idx_brands_ref_id ON brands (ref_id);
                CREATE INDEX idx_brands_type_id ON brands (type_id);
                CREATE INDEX idx_brands_id ON brands (id);
                CREATE INDEX idx_models_brand_id ON models (brand_id);
                CREATE INDEX idx_models_id ON models (id);
                CREATE INDEX idx_years_model_id ON years (model_id);
                CREATE INDEX idx_years_fuel_id ON years (fuel_id);
            "#
            }

            Sql::CreateConfig => {
                r#"
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

            Sql::SelectReferences => {
                r#"
                SELECT
                    id,
                    month,
                    year,
                    fipe
                FROM "references" r
                WHERE NOT EXISTS (
                    SELECT 1
                    FROM brands b
                    WHERE b.ref_id = r.id
                )
                "#
            }

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
                LEFT JOIN types t ON b.type_id = t.id
                WHERE NOT EXISTS (
                    SELECT 1
                    FROM models m
                    WHERE m.brand_id = b.id
                )
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
                    MAX(r.fipe) AS ref_id,
                    r."month" || '/' || r."year" ref_description,
                    b.type_id type_id,
                    t.description type_description
                FROM
                    models m
                JOIN brands b ON
                    m.brand_id = b.id
                JOIN "references" r ON
                    b.ref_id = r.id
                JOIN types t ON
                    b.type_id = t.id
                WHERE
                    NOT EXISTS (
                    SELECT
                        1
                    FROM
                        years y
                    WHERE
                        y.model_id = m.id
                )
                GROUP BY
                    m.fipe
            "#
            }
            Sql::SelectModelsReplicate => {
                r#"
                SELECT
                m.id,
                m.description,
                r."month" || '/' || r."year" ref_description
                FROM models m
                left join brands b on m.brand_id = b.id
                left join "references" r ON b.ref_id = r.id
                WHERE m.fipe = ?1
                AND NOT EXISTS (
                    SELECT 1
                    FROM years y
                    WHERE m.id = y.model_id
                )
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
                "INSERT INTO years (description, value, fipe, model_id, fuel_id) VALUES (?1, ?2, ?3, ?4, ?5)"
            }

            Sql::UpdateConfig => {
                "UPDATE config SET db_status = ?1, last_update = datetime('now', 'localtime')"
            }
        }
    }
}
