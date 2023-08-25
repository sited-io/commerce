use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::Pool;
use sea_query::{Asterisk, Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::api::peoplesmarkets::commerce::v1::OfferResponse;
use crate::db::DbError;

#[derive(Iden)]
#[iden(rename = "offers")]
pub enum OfferIden {
    Table,
    OfferId,
    MarketBoothId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Name,
    Description,
}

#[derive(Debug, Clone)]
pub struct Offer {
    pub offer_id: Uuid,
    pub market_booth_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
}

impl Offer {
    pub async fn create(
        pool: &Pool,
        market_booth_id: Uuid,
        user_id: &String,
        name: String,
        description: Option<String>,
    ) -> Result<Self, DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(OfferIden::Table)
            .columns([
                OfferIden::MarketBoothId,
                OfferIden::UserId,
                OfferIden::Name,
                OfferIden::Description,
            ])
            .values([
                market_booth_id.into(),
                user_id.into(),
                name.into(),
                description.unwrap_or_default().into(),
            ])?
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = client.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn get(
        pool: &Pool,
        offer_id: &Uuid,
    ) -> Result<Option<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(OfferIden::Table)
            .and_where(Expr::col(OfferIden::OfferId).eq(*offer_id))
            .build_postgres(PostgresQueryBuilder);

        Ok(client
            .query_opt(sql.as_str(), &values.as_params())
            .await?
            .map(Self::from))
    }

    pub async fn list(
        pool: &Pool,
        market_booth_id: Option<Uuid>,
        user_id: Option<&String>,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::select();

            query.column(Asterisk).from(OfferIden::Table);

            if let Some(market_booth_id) = market_booth_id {
                query.and_where(
                    Expr::col(OfferIden::MarketBoothId).eq(market_booth_id),
                );
            }

            if let Some(user_id) = user_id {
                query.and_where(Expr::col(OfferIden::UserId).eq(user_id));
            }

            query
                .limit(limit)
                .offset(offset)
                .build_postgres(PostgresQueryBuilder)
        };

        let rows = client.query(sql.as_str(), &values.as_params()).await?;

        Ok(rows.iter().map(Self::from).collect())
    }

    pub async fn update(
        pool: &Pool,
        offer_id: &Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Self, DbError> {
        let client = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::update();
            query.table(OfferIden::Table);

            if let Some(name) = name {
                query.value(OfferIden::Name, name);
            }

            if let Some(description) = description {
                query.value(OfferIden::Description, description);
            }

            query
                .and_where(Expr::col(OfferIden::OfferId).eq(*offer_id))
                .returning_all();

            query.build_postgres(PostgresQueryBuilder)
        };

        let row = client.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn delete(pool: &Pool, offer_id: &Uuid) -> Result<(), DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::delete()
            .from_table(OfferIden::Table)
            .and_where(Expr::col(OfferIden::OfferId).eq(*offer_id))
            .build_postgres(PostgresQueryBuilder);

        client.execute(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }
}

impl From<Offer> for OfferResponse {
    fn from(offer: Offer) -> Self {
        Self {
            offer_id: offer.offer_id.to_string(),
            market_booth_id: offer.market_booth_id.to_string(),
            user_id: offer.user_id,
            created_at: offer.created_at.timestamp(),
            updated_at: offer.updated_at.timestamp(),
            name: offer.name,
            description: offer.description,
        }
    }
}

impl From<&Row> for Offer {
    fn from(row: &Row) -> Self {
        Self {
            offer_id: row.get("offer_id"),
            market_booth_id: row.get("market_booth_id"),
            user_id: row.get("user_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            name: row.get("name"),
            description: row.get("description"),
        }
    }
}

impl From<Row> for Offer {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}
