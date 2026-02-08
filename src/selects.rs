use crate::schema::{Brands, Models, ModelsReplicate, References, Types};
use crate::ui::{Label, Sql};
use rusqlite::{Connection, Result};
pub fn select_types(conn: &Connection) -> Result<Vec<Types>, Box<dyn std::error::Error>> {
    let mut stmt = match conn.prepare(Sql::SelectTypes.as_str()) {
        Ok(s) => s,

        Err(rusqlite::Error::SqliteFailure(e, Some(msg))) if msg.contains("no such table") => {
            Label::TableNotExist.log();
            return Err(Box::new(e));
        }
        Err(e) => {
            return Err(Box::new(e));
        }
    };

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

pub fn select_references(conn: &Connection) -> Result<Vec<References>, Box<dyn std::error::Error>> {
    let mut stmt = match conn.prepare(Sql::SelectReferences.as_str()) {
        Ok(s) => s,

        Err(rusqlite::Error::SqliteFailure(e, Some(msg))) if msg.contains("no such table") => {
            Label::TableNotExist.log();
            return Err(Box::new(e));
        }
        Err(e) => {
            return Err(Box::new(e));
        }
    };
    let reference_iter = stmt.query_map([], |row| {
        Ok(References {
            id: row.get(0)?,
            month: row.get(1)?,
            year: row.get(2)?,
            fipe: row.get(3)?,
        })
    })?;

    let mut references = Vec::new();
    for reference in reference_iter {
        references.push(reference?);
    }
    Ok(references)
}

// Brands

pub fn select_brands(conn: &Connection) -> Result<Vec<Brands>, Box<dyn std::error::Error>> {
    let mut stmt = match conn.prepare(Sql::SelectBrands.as_str()) {
        Ok(s) => s,

        Err(rusqlite::Error::SqliteFailure(e, Some(msg))) if msg.contains("no such table") => {
            Label::TableNotExist.log();
            return Err(Box::new(e));
        }
        Err(e) => {
            return Err(Box::new(e));
        }
    };

    let brand_iter = stmt.query_map([], |row| {
        Ok(Brands {
            id: row.get(0)?,
            description: row.get(1)?,
            fipe: row.get(2)?,
            ref_id: row.get(3)?,
            ref_description: row.get(4)?,
            type_id: row.get(5)?,
            type_description: row.get(6)?,
        })
    })?;

    let mut brands = Vec::new();
    for brand in brand_iter {
        brands.push(brand?);
    }
    Ok(brands)
}

// Models

pub fn select_models(conn: &Connection) -> Result<Vec<Models>, Box<dyn std::error::Error>> {
    let mut stmt = match conn.prepare(Sql::SelectModels.as_str()) {
        Ok(s) => s,

        Err(rusqlite::Error::SqliteFailure(e, Some(msg))) if msg.contains("no such table") => {
            Label::TableNotExist.log();
            return Err(Box::new(e));
        }
        Err(e) => {
            return Err(Box::new(e));
        }
    };

    let model_iter = stmt.query_map([], |row| {
        Ok(Models {
            id: row.get("id")?,
            description: row.get("description")?,
            fipe: row.get("fipe")?,
            ref_id: row.get("ref_id")?,
            ref_description: row.get("ref_description")?,
            type_id: row.get("type_id")?,
            type_description: row.get("type_description")?,
            brand_id: row.get("brand_id")?,
            brand_description: row.get("brand_description")?,
        })
    })?;

    let mut models = Vec::new();
    for model in model_iter {
        models.push(model?);
    }
    Ok(models)
}

pub fn select_models_replicate(
    conn: &Connection,
    fipe: &str,
) -> Result<Vec<ModelsReplicate>, Box<dyn std::error::Error>> {
    let mut stmt = match conn.prepare(Sql::SelectModelsReplicate.as_str()) {
        Ok(s) => s,

        Err(rusqlite::Error::SqliteFailure(e, Some(msg))) if msg.contains("no such table") => {
            Label::TableNotExist.log();
            return Err(Box::new(e));
        }
        Err(e) => {
            return Err(Box::new(e));
        }
    };

    let model_iter = stmt.query_map([fipe], |row| {
        Ok(ModelsReplicate {
            id: row.get("id")?,
            description: row.get("description")?,
            ref_description: row.get("ref_description")?,
        })
    })?;

    let mut models = Vec::new();
    for model in model_iter {
        models.push(model?);
    }
    Ok(models)
}
