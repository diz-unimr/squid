use crate::api::error::ApiError;
use crate::api::server::ApiContext;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use entity::cert::{Entity, Model as Cert};
use sea_orm::EntityTrait;
use std::sync::Arc;

pub(crate) fn router() -> Router<Arc<ApiContext>> {
    Router::new()
        // routes
        .route("/certs", get(all))
}

async fn all(State(ctx): State<Arc<ApiContext>>) -> Result<Json<Vec<Cert>>, ApiError> {
    let certs = Entity::find().all(&ctx.db).await?;

    Ok(Json(certs.into_iter().collect()))
}
