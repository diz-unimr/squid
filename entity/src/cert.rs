use sea_orm::entity::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "certs")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_serializing)]
    pub id: i32,
    #[sea_orm(unique, index)]
    pub name: String,
    pub valid_from: DateTimeUtc,
    pub valid_to: DateTimeUtc,
    pub updated: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
