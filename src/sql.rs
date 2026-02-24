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
    SelectAllReferences,
    SelectReferences,
    SelectBrands,
    SelectModels,
    SelectModelsReplicate,
    SelectStatus,
    SelectCount { entity: String },
    SelectRowCount,

    // inserts / updates
    InsertReference,
    InsertBrand,
    InsertModel,
    InsertYear,
    UpdateStatus,
    UpdateRowCount { entity: String },
}

impl Sql {
    pub fn get(&self) -> String {
        match self {
            Sql::DropTables =>
                r#"
              DROP TABLE IF EXISTS config;
              DROP TABLE IF EXISTS years;
              DROP TABLE IF EXISTS models;
              DROP TABLE IF EXISTS brands;
              DROP TABLE IF EXISTS "references";
              DROP TABLE IF EXISTS fuels;
              DROP TABLE IF EXISTS types;
          "#.to_string(),

            Sql::CreateYears =>
                r#"
              CREATE TABLE years(
                  id integer PRIMARY KEY,
                  description text,
                  value date,
                  fipe text,
                  model_id integer,
                  fuel_id integer,
                  foreign key(model_id) references models(id),
                  foreign key(fuel_id) references fuels(id),
                  unique(fipe, model_id)
              )
          "#.to_string(),

            Sql::CreateModels =>
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
          "#.to_string(),

            Sql::CreateBrands =>
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
          "#.to_string(),

            Sql::CreateReferences =>
                r#"
              CREATE TABLE "references"(
                  id integer PRIMARY KEY,
                  ref_date date,
                  fipe text unique
              )
          "#.to_string(),

            Sql::CreateTypes =>
                r#"
              CREATE TABLE types(
                  id integer PRIMARY KEY,
                  description text
              )
          "#.to_string(),

            Sql::CreateFuels =>
                r#"
              CREATE TABLE fuels(
                  id integer PRIMARY KEY,
                  description text
              )
          "#.to_string(),

            Sql::InitTypes => "INSERT INTO types(description) VALUES (?1), (?2), (?3)".to_string(),

            Sql::InitFuels =>
                "INSERT INTO fuels(description) VALUES (?1), (?2), (?3), (?4), (?5), (?6), (?7), (?8)".to_string(),

            Sql::CreateIndexes =>
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
          "#.to_string(),

            Sql::CreateConfig =>
                r#"
              CREATE TABLE config(
                  db_status text,
                  last_update date,
                  brands_rowcount integer default 51500,
                  models_rowcount integer default 1970128,
                  years_rowcount integer default 8119581,
                  vehicles_rowcount integer default 0
              );

              INSERT INTO config(db_status, last_update) VALUES ('empty', datetime('now', 'localtime'));

              CREATE TRIGGER config_single_row
              BEFORE INSERT ON config
              WHEN (SELECT COUNT(*) FROM config) >= 1
              BEGIN
                  SELECT RAISE(ABORT, 'config can only have a single row.');
              END;
          "#.to_string(),

            Sql::SelectTypes => "SELECT id, description FROM types".to_string(),
            Sql::SelectAllReferences =>
                r#"
              SELECT
                  id,
                  ref_date,
                  fipe
              FROM "references"
              "#.to_string(),
            Sql::SelectReferences =>
                r#"
              SELECT
                  id,
                  ref_date,
                  fipe
              FROM "references" r
              WHERE NOT EXISTS (
                  SELECT 1
                  FROM brands b
                  WHERE b.ref_id = r.id
              )
              "#.to_string(),

            Sql::SelectBrands =>
                r#"
              SELECT
                  b.id AS id,
                  b.description AS description,
                  b.fipe AS fipe,
                  r.id AS ref_id,
                  CASE strftime('%m', r.ref_date)
                    WHEN '01' THEN 'janeiro'
                    WHEN '02' THEN 'fevereiro'
                    WHEN '03' THEN 'março'
                    WHEN '04' THEN 'abril'
                    WHEN '05' THEN 'maio'
                    WHEN '06' THEN 'junho'
                    WHEN '07' THEN 'julho'
                    WHEN '08' THEN 'agosto'
                    WHEN '09' THEN 'setembro'
                    WHEN '10' THEN 'outubro'
                    WHEN '11' THEN 'novembro'
                    WHEN '12' THEN 'dezembro'
                  END || '/' || strftime('%Y', r.ref_date) AS ref_date,
                  b.type_id AS type_id,
                  t.description type_description
              FROM brands b
              LEFT JOIN "references" r ON b.ref_id = r.id
              LEFT JOIN types t ON b.type_id = t.id
              WHERE NOT EXISTS (
                  SELECT 1
                  FROM models m
                  WHERE m.brand_id = b.id
              )
          "#.to_string(),

            Sql::SelectModels =>
                r#"
              SELECT
                  m.id AS id,
                  m.description AS description,
                  m.fipe AS fipe,
                  b.fipe AS brand_id,
                  b.description AS brand_description,
                  MAX(r.fipe) AS ref_id,
                  CASE strftime('%m', r.ref_date)
                    WHEN '01' THEN 'janeiro'
                    WHEN '02' THEN 'fevereiro'
                    WHEN '03' THEN 'março'
                    WHEN '04' THEN 'abril'
                    WHEN '05' THEN 'maio'
                    WHEN '06' THEN 'junho'
                    WHEN '07' THEN 'julho'
                    WHEN '08' THEN 'agosto'
                    WHEN '09' THEN 'setembro'
                    WHEN '10' THEN 'outubro'
                    WHEN '11' THEN 'novembro'
                    WHEN '12' THEN 'dezembro'
                  END || '/' || strftime('%Y', r.ref_date) AS ref_date,
                  b.type_id AS type_id,
                  t.description AS type_description
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
          "#.to_string(),

            Sql::SelectModelsReplicate =>
                r#"
              SELECT
              m.id AS id,
              m.description AS description,
              CASE strftime('%m', r.ref_date)
                WHEN '01' THEN 'janeiro'
                WHEN '02' THEN 'fevereiro'
                WHEN '03' THEN 'março'
                WHEN '04' THEN 'abril'
                WHEN '05' THEN 'maio'
                WHEN '06' THEN 'junho'
                WHEN '07' THEN 'julho'
                WHEN '08' THEN 'agosto'
                WHEN '09' THEN 'setembro'
                WHEN '10' THEN 'outubro'
                WHEN '11' THEN 'novembro'
                WHEN '12' THEN 'dezembro'
              END || '/' || strftime('%Y', r.ref_date) AS ref_date
              FROM models m
              LEFT JOIN brands b on m.brand_id = b.id
              LEFT JOIN "references" r ON b.ref_id = r.id
              WHERE m.fipe = ?1
              AND NOT EXISTS (
                  SELECT 1
                  FROM years y
                  WHERE m.id = y.model_id
              )
          "#.to_string(),

            Sql::SelectStatus => "SELECT db_status, last_update FROM config".to_string(),

            Sql::SelectCount { entity } => format!("SELECT count(id) FROM {}", entity),

            Sql::SelectRowCount =>
                "SELECT
              brands_rowcount,
              models_rowcount,
              years_rowcount,
              vehicles_rowcount
          FROM config".to_string(),

            Sql::InsertReference =>
                "INSERT INTO \"references\" (ref_date, fipe) VALUES (?1, ?2)".to_string(),

            Sql::InsertBrand =>
                "INSERT INTO brands (description, fipe, type_id, ref_id) VALUES (?1, ?2, ?3, ?4)".to_string(),

            Sql::InsertModel =>
                "INSERT INTO models (description, fipe, brand_id) VALUES (?1, ?2, ?3)".to_string(),

            Sql::InsertYear =>
                "INSERT INTO years (description, value, fipe, model_id, fuel_id) VALUES (?1, ?2, ?3, ?4, ?5)".to_string(),

            Sql::UpdateStatus =>
                "UPDATE config SET db_status = ?1, last_update = datetime('now', 'localtime')".to_string(),

            Sql::UpdateRowCount { entity } => format!("UPDATE config SET {}_rowcount = ?1", entity),
        }
    }
}
