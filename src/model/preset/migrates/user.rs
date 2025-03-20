// src/migrator/m20220602_000001_create_bakery_table.rs (create new file)

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "migration_presets_user"
    }
}

fn define_user_table() -> TableCreateStatement {
    Table::create().table(User::Table).col(
        ColumnDef::new(User::Id).integer().not_null().auto_increment().primary_key()
    )
        .col(ColumnDef::new(User::Name).string().not_null()).to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(define_user_table().into()).await
    }

    #[allow(deprecated)]
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(User::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum User {
    Table,
    Id,
    Name,
}
