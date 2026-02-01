use crate::schema::{ ReferencesResponse, ModelsResponse, FipeStruct };
use crate::selects::{ select_types, select_references, select_brands, select_models };
use crate::utils::insert_error;
use crate::ui::{ Label, Sql };
use reqwest::Client;
use rusqlite::{ params, Connection, Result };
use std::process::exit;
pub async fn load_references(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let response = match
        Client::new()
            .post("https://veiculos.fipe.org.br/api/veiculos/ConsultarTabelaDeReferencia")
            .header("Referer", "http://veiculos.fipe.org.br/")
            .header("Content-Type", "application/json")
            .send().await
    {
        Ok(r) => r,
        Err(e) => {
            let err_msg = e.to_string();
            (Label::ResponseError { message: &err_msg }).log();
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

    for r in &references {
        let cod_str = r.codigo.to_string();
        (Label::InsertReference { codigo: &cod_str, mes: &r.mes }).log();
        conn.execute(
            Sql::InsertReference.as_str(),
            params![r.mes.split('/').nth(0), r.mes.split('/').nth(1), r.codigo]
        )?;
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
                "codigoTipoVeiculo": t.id,
                "codigoTabelaReferencia": r.fipe
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
                    let err_msg = e.to_string();
                    (Label::ResponseError { message: &err_msg }).log();
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
            for b in brands {
                (Label::InsertBrand {
                    tipo: &t.description,
                    referencia: format!("{}/{}", &r.month, &r.year).as_str(),
                    marca: &b.label,
                    codigo: &b.value,
                }).log();
                conn.execute(Sql::InsertBrand.as_str(), params![b.label, b.value, t.id, r.id])?;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
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
                let err_msg = e.to_string();
                (Label::ResponseError { message: &err_msg }).log();
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
        for m in models.model {
            (Label::InsertModel {
                tipo: &b.type_description,
                referencia: &b.ref_description,
                marca: &b.description,
                modelo: &m.label,
                codigo: &m.value.to_string(),
            }).log();

            conn.execute(Sql::InsertModel.as_str(), params![m.label, m.value, b.id])?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
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
                let err_msg = e.to_string();
                (Label::ResponseError { message: &err_msg }).log();
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
        for y in years {
            (Label::InsertYear {
                tipo: &m.type_description,
                referencia: &m.ref_description,
                marca: &m.brand_description,
                modelo: &m.description,
                ano: &y.label,
                codigo: &y.value,
            }).log();
            conn.execute(Sql::InsertYear.as_str(), params![y.label, y.value, m.id])?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
    (Label::LoadOk { entity: "Years" }).log();
    Ok(())
}
