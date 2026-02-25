use crate::config::select_rowcount;
use crate::schema::{ ReferencesResponse, ModelsResponse, FipeStruct };
use crate::selects::{
    select_brands,
    select_models,
    select_models_replicate,
    select_all_references,
    select_references,
    select_types,
};
use crate::label::{ Label };
use crate::sql::{ Sql };
use crate::utils::{
    throttle,
    parse_date,
    progress_bar,
    parse_ref_date,
    get_random_user_agent,
    parse_year,
};
use reqwest::Client;
use rusqlite::{ params, Connection, Result };
use std::process::exit;
use std::sync::OnceLock;

static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

fn get_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(|| {
        Client::builder()
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .build()
            .expect(Label::ClientFail.to_string().as_str())
    })
}

async fn fetch_fipe(url: &str, body: &serde_json::Value) -> Option<reqwest::Response> {
    let client = get_client();
    loop {
        match
            client
                .post(url)
                .header("Referer", "http://veiculos.fipe.org.br/")
                .header("Content-Type", "application/json")
                .header("User-Agent", get_random_user_agent())
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

    let references_new: Vec<ReferencesResponse> = response.json().await?;
    let references_old = select_all_references(conn)?;
    let len: u64 = references_new.len().try_into().unwrap();
    let pb = progress_bar(len);
    let mut stmt = conn.prepare(Sql::InsertReference.get().as_str())?;
    for r in &references_new {
        if
            let Some(old) = references_old
                .iter()
                .find(|old| old.fipe.trim() == r.codigo.to_string().trim())
        {
            let mes_ano = parse_ref_date(old);
            pb.set_message(
                (Label::InsertReference {
                    codigo: &r.codigo.to_string(),
                    mes: mes_ano.as_str(),
                }).to_string()
            );
            pb.inc(1);
            continue;
        }
        let codigo = r.codigo.to_string();
        match stmt.execute(params![parse_date(&r.mes), r.codigo]) {
            Ok(_) => {
                pb.set_message(
                    (Label::InsertReference {
                        codigo: &r.codigo.to_string(),
                        mes: &r.mes,
                    }).to_string()
                );
                pb.inc(1);
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
    pb.finish_with_message((Label::LoadOk { entity: "References" }).to_string());
    Ok(())
}

pub async fn load_brands(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let count: u64 = select_rowcount(conn)?.brands_rowcount.try_into().unwrap();
    let pb = progress_bar(count);

    let types = match select_types(conn) {
        Ok(vt) => vt,
        Err(e) => {
            panic!("SQLITE_PREPARE_ERROR: {:?}", e);
        }
    };

    let references = match select_references(conn) {
        Ok(vr) => vr,
        Err(e) => {
            panic!("SQLITE_PREPARE_ERROR: {:?}", e);
        }
    };

    if references.len() == 0 || types.len() == 0 {
        Label::NoResults.log();
        return Ok(());
    }

    let url = "https://veiculos.fipe.org.br/api/veiculos/ConsultarMarcas";
    let mut stmt = conn.prepare(Sql::InsertBrand.get().as_str())?;
    for t in &types {
        for r in &references {
            let body =
                serde_json::json!({
                "codigoTipoVeiculo": &t.id,
                "codigoTabelaReferencia": &r.fipe
            });

            let response = fetch_fipe(url, &body).await.unwrap();

            let brands: Vec<FipeStruct> = match response.json().await {
                Ok(data) => data,
                Err(_) => {
                    println!("Decode error. Skipping...");
                    continue;
                }
            };
            for b in brands {
                match stmt.execute(params![b.label, b.value, t.id, r.id]) {
                    Ok(_) => {
                        let mes_ano = parse_ref_date(&r);
                        pb.set_message(
                            (Label::InsertBrand {
                                tipo: &t.description,
                                referencia: mes_ano.as_str(),
                                marca: &b.label,
                                codigo: &b.value,
                            }).to_string()
                        );
                        pb.inc(1);
                        ();
                    }

                    Err(rusqlite::Error::SqliteFailure(e, _)) if
                        e.code == rusqlite::ErrorCode::ConstraintViolation
                    => {
                        pb.set_message((Label::UniqueConstraint { fipe: &b.value }).to_string());
                    }

                    Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if
                        msg.contains("no such table")
                    => {
                        pb.set_message(Label::TableNotExist.to_string());
                        return Ok(());
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        pb.set_message((Label::ResponseError { message: &err_msg }).to_string());
                        exit(1);
                    }
                };
            }
            throttle().await;
        }
    }
    pb.finish_with_message((Label::LoadOk { entity: "Brands" }).to_string());
    Ok(())
}

pub async fn load_models(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let count: u64 = select_rowcount(conn)?.models_rowcount.try_into().unwrap();
    let pb = progress_bar(count);

    let brands = match select_brands(conn) {
        Ok(vb) => vb,
        Err(e) => {
            panic!("{:?}", e);
        }
    };
    if brands.len() == 0 {
        (Label::LoadOk { entity: "Models" }).log();
        return Ok(());
    }
    let url = "https://veiculos.fipe.org.br/api/veiculos/ConsultarModelos";
    let mut stmt = conn.prepare(Sql::InsertModel.get().as_str())?;
    for b in &brands {
        let body =
            serde_json::json!({
            "codigoTipoVeiculo": b.type_id,
            "codigoTabelaReferencia": b.ref_id,
            "codigoMarca": b.fipe
        });
        let response = fetch_fipe(&url, &body).await.unwrap();

        let models: ModelsResponse = match response.json().await {
            Ok(data) => data,
            Err(_) => {
                println!("Decode error. Skipping...");
                continue;
            }
        };
        for m in models.model {
            match stmt.execute(params![m.label, m.value, b.id]) {
                Ok(_) => {
                    pb.inc(1);
                    pb.set_message(
                        (Label::InsertModel {
                            tipo: &b.type_description,
                            referencia: &b.ref_date,
                            marca: &b.description,
                            modelo: &m.label,
                            codigo: &m.value.to_string(),
                        }).to_string()
                    );
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
    pb.finish_with_message((Label::LoadOk { entity: "Models" }).to_string());
    Ok(())
}

pub async fn load_years(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let count: u64 = select_rowcount(conn)?.years_rowcount.try_into().unwrap();
    let pb = progress_bar(count);

    let models = match select_models(conn) {
        Ok(vm) => vm,
        Err(e) => {
            panic!("{:?}", e);
        }
    };
    if models.len() == 0 {
        (Label::LoadOk { entity: "Years" }).log();
        return Ok(());
    }
    let url = "https://veiculos.fipe.org.br/api/veiculos/ConsultarAnoModelo";
    let mut stmt = conn.prepare(Sql::InsertYear.get().as_str())?;
    for m in &models {
        let body =
            serde_json::json!({
            "codigoTipoVeiculo": &m.type_id,
            "codigoTabelaReferencia": &m.ref_id,
            "codigoMarca": &m.brand_id,
            "codigoModelo": &m.fipe
        });
        let response = fetch_fipe(&url, &body).await.unwrap();

        let years: Vec<FipeStruct> = match response.json().await {
            Ok(data) => data,
            Err(_) => {
                println!("Decode error. Skipping...");
                continue;
            }
        };

        let models_replica = select_models_replicate(conn, &m.fipe)?;
        for y in years {
            let (year_date, fuel_id) = parse_year(&y.value);
            for mr in &models_replica {
                match stmt.execute(params![y.label, year_date, y.value, mr.id, fuel_id]) {
                    Ok(_) => {
                        pb.inc(1);
                        pb.set_message(
                            (Label::InsertYear {
                                tipo: &m.type_description,
                                referencia: &mr.ref_date,
                                marca: &m.brand_description,
                                modelo: &mr.description,
                                ano: &y.label,
                            }).to_string()
                        );
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
    pb.finish_with_message((Label::LoadOk { entity: "Years" }).to_string());
    Ok(())
}
