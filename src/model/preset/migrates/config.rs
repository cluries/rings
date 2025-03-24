use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "migration_presets_config"
    }
}

fn define_config_table() -> TableCreateStatement {
    Table::create()
        .table(Config::Table)
        .col(ColumnDef::new(Config::Id).integer().not_null().auto_increment().primary_key())
        .col(ColumnDef::new(Config::Name).string().not_null())
        .col(ColumnDef::new(Config::Version).integer().not_null())
        .col(ColumnDef::new(Config::Value).string().not_null())
        .col(ColumnDef::new(Config::Mark).string().not_null())
        .col(ColumnDef::new(Config::State).integer().not_null())
        .col(ColumnDef::new(Config::UpdatedAt).date_time().not_null())
        .col(ColumnDef::new(Config::CreatedAt).date_time().not_null())
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(define_config_table().into()).await
    }

    #[allow(deprecated)]
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Config::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum Config {
    Table,
    Id,
    Name,
    Version,
    Value,
    Mark,
    State,
    UpdatedAt,
    CreatedAt,
}
