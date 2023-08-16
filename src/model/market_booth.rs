use chrono::{DateTime, Utc};
use deadpool_postgres::{tokio_postgres::Row, Pool};
use sea_query::{Asterisk, Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::api::peoplesmarkets::commerce::v1::MarketBoothResponse;
use crate::db::DbError;

#[derive(Iden)]
pub enum MarketBooths {
    Table,
    MarketBoothId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Name,
    Description,
}

#[derive(Debug, Clone)]
pub struct MarketBooth {
    pub market_booth_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
}

impl MarketBooth {
    pub async fn create(
        pool: &Pool,
        user_id: &String,
        name: String,
        description: Option<String>,
    ) -> Result<Self, DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(MarketBooths::Table)
            .columns([
                MarketBooths::UserId,
                MarketBooths::Name,
                MarketBooths::Description,
            ])
            .values([
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
        market_booth_id: &Uuid,
    ) -> Result<Option<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(MarketBooths::Table)
            .and_where(
                Expr::col(MarketBooths::MarketBoothId).eq(*market_booth_id),
            )
            .build_postgres(PostgresQueryBuilder);

        Ok(client
            .query_opt(sql.as_str(), &values.as_params())
            .await?
            .map(Self::from))
    }

    pub async fn list(
        pool: &Pool,
        user_id: Option<&String>,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::select();

            query.column(Asterisk).from(MarketBooths::Table);

            if let Some(user_id) = user_id {
                query.and_where(Expr::col(MarketBooths::UserId).eq(user_id));
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
        market_booth_id: &Uuid,
        name: Option<&String>,
        description: Option<&String>,
    ) -> Result<Self, DbError> {
        let client = pool.get().await?;

        let mut query = Query::update();
        query.table(MarketBooths::Table);

        if let Some(name) = name {
            query.value(MarketBooths::Name, name);
        }

        if let Some(description) = description {
            query.value(MarketBooths::Description, description);
        }

        query
            .and_where(
                Expr::col(MarketBooths::MarketBoothId).eq(*market_booth_id),
            )
            .returning_all();

        let (sql, values) = query.build_postgres(PostgresQueryBuilder);

        Ok(Self::from(
            client.query_one(sql.as_str(), &values.as_params()).await?,
        ))
    }

    pub async fn delete(
        pool: &Pool,
        market_booth_id: &Uuid,
    ) -> Result<(), DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::delete()
            .from_table(MarketBooths::Table)
            .and_where(
                Expr::col(MarketBooths::MarketBoothId).eq(*market_booth_id),
            )
            .build_postgres(PostgresQueryBuilder);

        client.execute(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }
}

impl Into<MarketBoothResponse> for MarketBooth {
    fn into(self) -> MarketBoothResponse {
        MarketBoothResponse {
            market_booth_id: self.market_booth_id.to_string(),
            user_id: self.user_id,
            created_at: self.created_at.timestamp(),
            updated_at: self.updated_at.timestamp(),
            name: self.name,
            description: self.description,
        }
    }
}

impl From<&Row> for MarketBooth {
    fn from(row: &Row) -> Self {
        Self {
            market_booth_id: row.get("market_booth_id"),
            user_id: row.get("user_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            name: row.get("name"),
            description: row.get("description"),
        }
    }
}

impl From<Row> for MarketBooth {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}
