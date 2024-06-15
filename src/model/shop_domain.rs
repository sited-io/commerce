use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::{Pool, Transaction};
use sea_query::{Asterisk, Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::api::sited_io::commerce::v1::DomainStatus;
use crate::db::DbError;

#[derive(Debug, Clone, Copy, Iden)]
#[iden(rename = "shop_domains")]
pub enum ShopDomainIden {
    Table,
    ShopId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Domain,
    Status,
    ClientId,
}

#[derive(Debug, Clone)]
pub struct ShopDomain {
    pub shop_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub domain: String,
    pub status: String,
    pub client_id: Option<String>,
}

impl ShopDomain {
    pub async fn create(
        pool: &Pool,
        user_id: &String,
        shop_id: &Uuid,
        domain: &String,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(ShopDomainIden::Table)
            .columns([
                ShopDomainIden::UserId,
                ShopDomainIden::ShopId,
                ShopDomainIden::Domain,
                ShopDomainIden::Status,
            ])
            .values([
                user_id.into(),
                (*shop_id).into(),
                domain.into(),
                DomainStatus::Pending.as_str_name().into(),
            ])?
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn get(
        pool: &Pool,
        shop_id: &Uuid,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(ShopDomainIden::Table)
            .and_where(Expr::col(ShopDomainIden::ShopId).eq(*shop_id))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn get_by_domain(
        pool: &Pool,
        domain: &String,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(ShopDomainIden::Table)
            .and_where(Expr::col(ShopDomainIden::Domain).eq(domain))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn get_for_user(
        pool: &Pool,
        user_id: &String,
        shop_id: &Uuid,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(ShopDomainIden::Table)
            .and_where(Expr::col(ShopDomainIden::UserId).eq(user_id))
            .and_where(Expr::col(ShopDomainIden::ShopId).eq(*shop_id))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn update<'a>(
        transaction: &Transaction<'a>,
        shop_id: &Uuid,
        domain: &String,
        status: &String,
        client_id: &String,
    ) -> Result<Self, DbError> {
        let (sql, values) = Query::update()
            .table(ShopDomainIden::Table)
            .value(ShopDomainIden::Status, status)
            .value(ShopDomainIden::ClientId, client_id)
            .and_where(Expr::col(ShopDomainIden::ShopId).eq(*shop_id))
            .and_where(Expr::col(ShopDomainIden::Domain).eq(domain))
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = transaction
            .query_one(sql.as_str(), &values.as_params())
            .await?;

        Ok(Self::from(row))
    }

    pub async fn delete<'a>(
        transaction: &Transaction<'a>,
        user_id: &String,
        shop_id: &Uuid,
        domain: &String,
    ) -> Result<(), DbError> {
        let (sql, values) = Query::delete()
            .from_table(ShopDomainIden::Table)
            .and_where(Expr::col(ShopDomainIden::UserId).eq(user_id))
            .and_where(Expr::col(ShopDomainIden::ShopId).eq(*shop_id))
            .and_where(Expr::col(ShopDomainIden::Domain).eq(domain))
            .build_postgres(PostgresQueryBuilder);

        transaction.query(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }
}

impl From<&Row> for ShopDomain {
    fn from(row: &Row) -> Self {
        Self {
            shop_id: row.get(ShopDomainIden::ShopId.to_string().as_str()),
            user_id: row.get(ShopDomainIden::UserId.to_string().as_str()),
            created_at: row.get(ShopDomainIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(ShopDomainIden::UpdatedAt.to_string().as_str()),
            domain: row.get(ShopDomainIden::Domain.to_string().as_str()),
            status: row.get(ShopDomainIden::Status.to_string().as_str()),
            client_id: row.get(ShopDomainIden::ClientId.to_string().as_str()),
        }
    }
}

impl From<Row> for ShopDomain {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}
