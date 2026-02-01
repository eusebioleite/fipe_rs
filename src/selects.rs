use crate::schema::{Brands, Models, References, Types};
use crate::ui::Sql;
use rusqlite::{Connection, Result};
pub fn select_types(conn: &Connection) -> Result<Vec<Types>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(Sql::SelectTypes.as_str())?;
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
    let mut stmt = conn.prepare(Sql::SelectReferences.as_str())?;
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
    let mut stmt = conn.prepare(Sql::SelectBrands.as_str())?;
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
    let mut stmt = conn.prepare(Sql::SelectModels.as_str())?;
    let model_iter = stmt.query_map([], |row| {
        Ok(Models {
            id: row.get(0)?,
            description: row.get(1)?,
            fipe: row.get(1)?,
            ref_id: row.get(1)?,
            ref_description: row.get(1)?,
            type_id: row.get(1)?,
            type_description: row.get(1)?,
            brand_id: row.get(1)?,
            brand_description: row.get(1)?,
        })
    })?;

    let mut models = Vec::new();
    for model in model_iter {
        models.push(model?);
    }
    Ok(models)
}
