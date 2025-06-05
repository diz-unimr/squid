use crate::sea_orm::Schema;
use entity::cert;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let builder = db.get_database_backend();
        let schema = Schema::new(builder);

        let stmt = builder.build(&schema.create_table_from_entity(cert::Entity));

        match db.execute(stmt).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}
