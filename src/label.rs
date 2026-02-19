use owo_colors::OwoColorize;
use std::fmt;

pub enum Label<'a> {
    // main
    Header {
        db_status: &'a str,
        last_update: &'a str,
    },

    // setup_db
    DbCreationOk,
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
    NoResults,
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
    },
    PressKeyContinue,
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
                "  {}:  {}",
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
                "  {}:  {} - {}",
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
            Label::NoResults => write!(
                f,
                "{}: {}",
                "[ERROR]".bold().bright_red(),
                "No results.".italic().black().dimmed()
            ),
            Label::InsertBrand {
                tipo,
                referencia,
                marca,
                codigo,
            } => {
                write!(
                    f,
                    "{}:  {} | {} | {} - {}",
                    "[SUCCESS]".bold().bright_green(),
                    tipo.bold().blue(),
                    referencia.bold().yellow(),
                    marca,
                    codigo
                )
            }
            Label::InsertModel {
                tipo,
                referencia,
                marca,
                modelo,
                codigo,
            } => {
                write!(
                    f,
                    "   {}:  {} | {} | {} | {} - {}",
                    "[SUCCESS]".bold().bright_green(),
                    tipo.bold().blue(),
                    referencia.bold().yellow(),
                    marca.bold().red(),
                    modelo,
                    codigo
                )
            }
            Label::InsertYear {
                tipo,
                referencia,
                marca,
                modelo,
                ano,
            } => {
                write!(
                    f,
                    "   {}:  {} | {} | {} | {} | {}",
                    "[SUCCESS]".bold().bright_green(),
                    tipo.bold().blue(),
                    referencia.bold().yellow(),
                    marca.bold().red(),
                    modelo.bold().magenta(),
                    ano
                )
            }
            Label::PressKeyContinue => write!(
                f,
                "{}",
                "Press any key to continue...".italic().black().dimmed()
            ),
            Label::DbCreationOk => write!(
                f,
                "  {}:  {}",
                "[SUCCESS]".bold().bright_green(),
                "Database successfully created.".italic()
            ),
        }
    }
}

impl<'a> Label<'a> {
    pub fn log(&self) {
        println!("{}", self);
    }
}
