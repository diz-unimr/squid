use sea_orm::entity::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "certs")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub name: String,
    pub alias: String,
    pub valid_from: DateTime,
    pub valid_to: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}