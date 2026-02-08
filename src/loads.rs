use crate::schema::{ ReferencesResponse, ModelsResponse, FipeStruct };
use crate::selects::{
    select_brands,
    select_models,
    select_models_replicate,
    select_references,
    select_types,
};
use crate::ui::{ Label, Sql };
use crate::utils::{ throttle };
use reqwest::Client;
use rusqlite::types::Null;
use rusqlite::{ params, Connection, Result };
use std::process::exit;

async fn fetch_fipe(
    client: &reqwest::Client,
    url: &str,
    body: &serde_json::Value
) -> Option<reqwest::Response> {
    loop {
        match
            client
                .post(url)
                .header("Referer", "http://veiculos.fipe.org.br/")
                .header("Content-Type", "application/json")
                .header(
                    "User-Agent",
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
                )
                .json(body)
                .send().await
        {
            Ok(res) if res.status().is_success() => {
                return Some(res);
            }
            Ok(res) => {
                (Label::ApiBlock { code: res.status().as_str() }).log();
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
            Err(e) => {
                let err_msg = e.to_string();
                (Label::ApiConnectionError { message: &err_msg }).log();
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

pub async fn load_references(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let response = Client::new()
        .post("https://veiculos.fipe.org.br/api/veiculos/ConsultarTabelaDeReferencia")
        .header("Referer", "http://veiculos.fipe.org.br/")
        .header("Content-Type", "application/json")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        )

        .send().await?;

    let references: Vec<ReferencesResponse> = response.json().await?;

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

            Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if msg.contains("no such table") => {
                Label::TableNotExist.log();
                return Ok(());
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
    let types = match select_types(conn) {
        Ok(vt) => vt,
        Err(e) => {
            return Ok(());
        }
    };
    let references = match select_references(conn) {
        Ok(vr) => vr,
        Err(e) => {
            return Ok(());
        }
    };

    if references.len() == 0 {
        (Label::LoadOk { entity: "Brands" }).log();
        return Ok(());
    }
    let url = "https://veiculos.fipe.org.br/api/veiculos/ConsultarMarcas";
    let client = reqwest::Client
        ::builder()
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .build()?;

    for t in &types {
        for r in &references {
            let body =
                serde_json::json!({
                "codigoTipoVeiculo": &t.id,
                "codigoTabelaReferencia": &r.fipe
            });

            let response = fetch_fipe(&client, url, &body).await.unwrap();

            let brands: Vec<FipeStruct> = match response.json().await {
                Ok(data) => data,
                Err(_) => {
                    println!("Erro de decode inesperado. Provavelmente HTML de erro. Pulando...");
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

                    Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if
                        msg.contains("no such table")
                    => {
                        Label::TableNotExist.log();
                        return Ok(());
                    }

                    Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if
                        msg.contains("no such table")
                    => {
                        Label::TableNotExist.log();
                        return Ok(());
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        (Label::ResponseError { message: &err_msg }).log();
                        exit(1);
                    }
                };
            }
            throttle().await;
        }
    }
    (Label::LoadOk { entity: "Brands" }).log();
    Ok(())
}

pub async fn load_models(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let brands = match select_brands(conn) {
        Ok(vb) => vb,
        Err(e) => {
            return Ok(());
        }
    };
    if brands.len() == 0 {
        (Label::LoadOk { entity: "Models" }).log();
        return Ok(());
    }
    let url = "https://veiculos.fipe.org.br/api/veiculos/ConsultarModelos";
    let client = reqwest::Client
        ::builder()
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .build()?;
    for b in &brands {
        let body =
            serde_json::json!({
            "codigoTipoVeiculo": b.type_id,
            "codigoTabelaReferencia": b.ref_id,
            "codigoMarca": b.fipe
        });
        let response = fetch_fipe(&client, &url, &body).await.unwrap();

        let models: ModelsResponse = match response.json().await {
            Ok(data) => data,
            Err(_) => {
                println!("Erro de decode inesperado. Provavelmente HTML de erro. Pulando...");
                continue;
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

                Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if
                    msg.contains("no such table")
                => {
                    Label::TableNotExist.log();
                    return Ok(());
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    (Label::ResponseError { message: &err_msg }).log();
                    exit(1);
                }
            }
        }
        throttle().await;
    }
    (Label::LoadOk { entity: "Models" }).log();
    Ok(())
}

pub async fn load_years(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let models = match select_models(conn) {
        Ok(vm) => vm,
        Err(e) => {
            return Ok(());
        }
    };
    if models.len() == 0 {
        (Label::LoadOk { entity: "Years" }).log();
        return Ok(());
    }
    let url = "https://veiculos.fipe.org.br/api/veiculos/ConsultarAnoModelo";
    let client = reqwest::Client
        ::builder()
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .build()?;
    for m in &models {
        let body =
            serde_json::json!({
            "codigoTipoVeiculo": &m.type_id,
            "codigoTabelaReferencia": &m.ref_id,
            "codigoMarca": &m.brand_id,
            "codigoModelo": &m.fipe
        });
        let response = fetch_fipe(&client, &url, &body).await.unwrap();

        let years: Vec<FipeStruct> = match response.json().await {
            Ok(data) => data,
            Err(_) => {
                println!("Erro de decode inesperado. Provavelmente HTML de erro. Pulando...");
                continue;
            }
        };

        let models_replica = select_models_replicate(conn, &m.fipe)?;
        for y in years {
            let parts: Vec<&str> = y.value.split('-').collect();
            let value = parts[0];
            let fuel_id = parts.get(1);
            for mr in &models_replica {
                match
                    conn.execute(
                        Sql::InsertYear.as_str(),
                        params![y.label, value, y.value, mr.id, fuel_id]
                    )
                {
                    Ok(_) => {
                        (Label::InsertYear {
                            tipo: &m.type_description,
                            referencia: &mr.ref_description,
                            marca: &m.brand_description,
                            modelo: &mr.description,
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

                    Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if
                        msg.contains("no such table")
                    => {
                        Label::TableNotExist.log();
                        return Ok(());
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        (Label::ResponseError { message: &err_msg }).log();
                        exit(1);
                    }
                };
            }
        }
        throttle().await;
    }
    (Label::LoadOk { entity: "Years" }).log();
    Ok(())
}
