use sea_orm_migration::prelude::*;

use super::m20210101_000021_create_wiki::Wiki;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000022_create_wiki_related"
    }
}

#[derive(Iden)]
pub enum WikiRelated {
    Table,
    Id,
    WikiId,
    RelatedId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WikiRelated::Table)
                    .col(
                        ColumnDef::new(WikiRelated::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WikiRelated::WikiId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("wiki_related_wiki_id_fkey")
                            .from(WikiRelated::Table, WikiRelated::WikiId)
                            .to(Wiki::Table, Wiki::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(WikiRelated::RelatedId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("wiki_related_related_id_fkey")
                            .from(WikiRelated::Table, WikiRelated::RelatedId)
                            .to(Wiki::Table, Wiki::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WikiRelated::Table).to_owned())
            .await
    }
}
