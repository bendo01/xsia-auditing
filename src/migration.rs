//! Migration creating the `audits` table. Add [`Migration`] to your migrator:
//! `vec![Box::new(xsia_auditing::migration::Migration), ...]`
use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum Audits {
    Table,
    Id,
    UserType,
    UserId,
    Event,
    AuditableType,
    AuditableId,
    OldValues,
    NewValues,
    Url,
    IpAddress,
    UserAgent,
    Tags,
    CreatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260101_000000_create_audits_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Audits::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Audits::Id).big_integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Audits::UserType).string().null())
                    .col(ColumnDef::new(Audits::UserId).string().null())
                    .col(ColumnDef::new(Audits::Event).string().not_null())
                    .col(ColumnDef::new(Audits::AuditableType).string().not_null())
                    .col(ColumnDef::new(Audits::AuditableId).string().not_null())
                    .col(ColumnDef::new(Audits::OldValues).json().null())
                    .col(ColumnDef::new(Audits::NewValues).json().null())
                    .col(ColumnDef::new(Audits::Url).text().null())
                    .col(ColumnDef::new(Audits::IpAddress).string().null())
                    .col(ColumnDef::new(Audits::UserAgent).string().null())
                    .col(ColumnDef::new(Audits::Tags).string().null())
                    .col(ColumnDef::new(Audits::CreatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_audits_auditable")
                    .table(Audits::Table)
                    .col(Audits::AuditableType)
                    .col(Audits::AuditableId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_audits_user")
                    .table(Audits::Table)
                    .col(Audits::UserType)
                    .col(Audits::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Audits::Table).to_owned()).await
    }
}
