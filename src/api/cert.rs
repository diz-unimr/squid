use crate::api::error::ApiError;
use crate::api::server::ApiContext;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router, debug_handler};
use entity::cert::{ActiveModel, Entity, Model as Cert};
use sea_orm::prelude::DateTimeUtc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub(crate) fn router() -> Router<Arc<ApiContext>> {
    Router::new()
        // routes
        .route("/certs", get(all).post(create))
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct TlsInfo {
    name: String,
    alias: Vec<String>,
    valid_from: DateTimeUtc,
    valid_to: DateTimeUtc,
}

impl From<Cert> for TlsInfo {
    fn from(c: Cert) -> Self {
        TlsInfo {
            name: c.name,
            alias: c.alias.split(",").map(|s| s.to_owned()).collect(),
            valid_from: c.valid_from,
            valid_to: c.valid_to,
        }
    }
}

async fn all(State(ctx): State<Arc<ApiContext>>) -> Result<Json<Vec<TlsInfo>>, ApiError> {
    let certs = Entity::find().all(&ctx.db).await?;

    Ok(Json(certs.into_iter().map(Into::into).collect()))
}

#[debug_handler]
async fn create(
    State(ctx): State<Arc<ApiContext>>,
    tls: Json<TlsInfo>,
) -> Result<(StatusCode, Json<TlsInfo>), ApiError> {
    let cert: ActiveModel = ActiveModel {
        name: Set(tls.name.to_owned()),
        alias: Set(tls.alias.join(",")),
        valid_from: Set(tls.valid_from),
        valid_to: Set(tls.valid_to),
        ..Default::default()
    };

    let inserted: Cert = cert.insert(&ctx.db).await?;

    Ok((StatusCode::CREATED, Json(inserted.into())))
}
