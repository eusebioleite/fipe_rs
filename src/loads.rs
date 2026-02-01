use crate::schema::{ ReferencesResponse, ModelsResponse, FipeStruct };
use crate::selects::{ select_types, select_references, select_brands, select_models };
use crate::utils::insert_error;
use crate::ui::{ Label, Sql };
use reqwest::Client;
use rusqlite::{ params, Connection, Result };
use std::process::exit;
pub async fn load_references(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://veiculos.fipe.org.br/api/veiculos/ConsultarTabelaDeReferencia";
    let response = match
        Client::new()
            .post(url)
            .header("Referer", "http://veiculos.fipe.org.br/")
            .header("Content-Type", "application/json")
            .send().await
    {
        Ok(r) => r,
        Err(e) => {
            let err_msg = e.to_string();
            (Label::ResponseError { message: &err_msg }).log();
            insert_error(&conn, "References", &url, "", &err_msg).await?;
            exit(1);
        }
    };

    let references: Vec<ReferencesResponse> = match response.json().await {
        Ok(data) => data,
        Err(e) => {
            let err_msg = format!("JSON Parse Error: {}", e);
            (Label::ResponseError { message: &err_msg }).log();
            insert_error(&conn, "References", &url, "", &err_msg).await?;
            return Ok(());
        }
    };

    for r in &references {
        let codigo = r.codigo.to_string();
        match
            conn.execute(
                Sql::InsertReference.as_str(),
                params![r.mes.split('/').nth(0), r.mes.split('/').nth(1), r.codigo]
            )
        {
            Ok(_) => {
                (Label::InsertReference { codigo: &codigo, mes: &r.mes }).log();
                ();
            }

            Err(rusqlite::Error::SqliteFailure(e, _)) if
                e.code == rusqlite::ErrorCode::ConstraintViolation
            => {
                (Label::UniqueConstraint { fipe: &codigo }).log();
            }

            Err(e) => {
                let err_msg = e.to_string();
                (Label::ResponseError { message: &err_msg }).log();
                exit(1);
            }
        }
    }
    (Label::LoadOk { entity: "References" }).log();
    Ok(())
}

pub async fn load_brands(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let types = select_types(conn)?;
    let references = select_references(conn)?;

    for t in &types {
        for r in &references {
            let body =
                serde_json::json!({
                "codigoTipoVeiculo": &t.id,
                "codigoTabelaReferencia": &r.fipe
            });
            let url = "https://veiculos.fipe.org.br/api/veiculos/ConsultarMarcas";
            let response = match
                Client::new()
                    .post(url)
                    .header("Referer", "http://veiculos.fipe.org.br/")
                    .header("Content-Type", "application/json")
                    .body(body.to_string())
                    .send().await
            {
                Ok(r) => r,
                Err(e) => {
                    let err_msg = e.to_string();
                    (Label::ResponseError { message: &err_msg }).log();
                    insert_error(&conn, "Brands", &url, &body.to_string(), &err_msg).await?;
                    continue;
                }
            };

            let brands: Vec<FipeStruct> = match response.json().await {
                Ok(data) => data,
                Err(e) => {
                    let err_msg = format!("JSON Parse Error: {}", e);
                    (Label::ResponseError { message: &err_msg }).log();
                    insert_error(&conn, "Brands", &url, &body.to_string(), &err_msg).await?;
                    continue;
                }
            };
            for b in brands {
                match
                    conn.execute(Sql::InsertBrand.as_str(), params![b.label, b.value, t.id, r.id])
                {
                    Ok(_) => {
                        (Label::InsertBrand {
                            tipo: &t.description,
                            referencia: format!("{}/{}", &r.month, &r.year).as_str(),
                            marca: &b.label,
                            codigo: &b.value,
                        }).log();
                        ();
                    }

                    Err(rusqlite::Error::SqliteFailure(e, _)) if
                        e.code == rusqlite::ErrorCode::ConstraintViolation
                    => {
                        (Label::UniqueConstraint { fipe: &b.value }).log();
                    }

                    Err(e) => {
                        let err_msg = e.to_string();
                        (Label::ResponseError { message: &err_msg }).log();
                        exit(1);
                    }
                };
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
    (Label::LoadOk { entity: "Brands" }).log();
    Ok(())
}

pub async fn load_models(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let brands = select_brands(conn)?;
    for b in &brands {
        let body =
            serde_json::json!({
            "codigoTipoVeiculo": b.type_id,
            "codigoTabelaReferencia": b.ref_id,
            "codigoMarca": b.fipe
        });
        let url = "https://veiculos.fipe.org.br/api/veiculos/ConsultarModelos";
        let response = match
            Client::new()
                .post(url)
                .header("Referer", "http://veiculos.fipe.org.br/")
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .send().await
        {
            Ok(r) => r,
            Err(e) => {
                let err_msg = e.to_string();
                (Label::ResponseError { message: &err_msg }).log();
                insert_error(&conn, "Models", &url, &body.to_string(), &err_msg).await?;
                exit(1);
            }
        };
        let models: ModelsResponse = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                let err_msg = format!("JSON Parse Error: {}", e);
                (Label::ResponseError { message: &err_msg }).log();
                insert_error(&conn, "Models", &url, &body.to_string(), &err_msg).await?;
                return Ok(());
            }
        };
        for m in models.model {
            match conn.execute(Sql::InsertModel.as_str(), params![m.label, m.value, b.id]) {
                Ok(_) => {
                    (Label::InsertModel {
                        tipo: &b.type_description,
                        referencia: &b.ref_description,
                        marca: &b.description,
                        modelo: &m.label,
                        codigo: &m.value.to_string(),
                    }).log();
                    ();
                }

                Err(rusqlite::Error::SqliteFailure(e, _)) if
                    e.code == rusqlite::ErrorCode::ConstraintViolation
                => {
                    (Label::UniqueConstraint { fipe: &m.value.to_string() }).log();
                }

                Err(e) => {
                    let err_msg = e.to_string();
                    (Label::ResponseError { message: &err_msg }).log();
                    exit(1);
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
    (Label::LoadOk { entity: "Models" }).log();
    Ok(())
}

pub async fn load_years(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let models = select_models(conn)?;
    for m in &models {
        let body =
            serde_json::json!({
            "codigoTipoVeiculo": &m.type_id,
            "codigoTabelaReferencia": &m.ref_id,
            "codigoMarca": &m.brand_id,
            "codigoModelo": &m.fipe
        });
        let url = "https://veiculos.fipe.org.br/api/veiculos/ConsultarAnoModelo";
        let response = match
            Client::new()
                .post(url)
                .header("Referer", "http://veiculos.fipe.org.br/")
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .send().await
        {
            Ok(r) => r,
            Err(e) => {
                let err_msg = e.to_string();
                (Label::ResponseError { message: &err_msg }).log();
                insert_error(&conn, "Years", &url, &body.to_string(), &err_msg).await?;
                exit(1);
            }
        };

        let years: Vec<FipeStruct> = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                let err_msg = format!("JSON Parse Error: {}", e);
                (Label::ResponseError { message: &err_msg }).log();
                insert_error(&conn, "Years", &url, &body.to_string(), &err_msg).await?;
                return Ok(());
            }
        };
        for y in years {
            match conn.execute(Sql::InsertYear.as_str(), params![y.label, y.value, m.id]) {
                Ok(_) => {
                    (Label::InsertYear {
                        tipo: &m.type_description,
                        referencia: &m.ref_description,
                        marca: &m.brand_description,
                        modelo: &m.description,
                        ano: &y.label,
                        codigo: &y.value,
                    }).log();
                    ();
                }

                Err(rusqlite::Error::SqliteFailure(e, _)) if
                    e.code == rusqlite::ErrorCode::ConstraintViolation
                => {
                    (Label::UniqueConstraint { fipe: &y.value }).log();
                }

                Err(e) => {
                    let err_msg = e.to_string();
                    (Label::ResponseError { message: &err_msg }).log();
                    exit(1);
                }
            };
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
    (Label::LoadOk { entity: "Years" }).log();
    Ok(())
}
