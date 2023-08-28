use chrono::{DateTime, Utc};
use deadpool_postgres::{tokio_postgres::Row, Pool};
use sea_query::extension::postgres::PgExpr;
use sea_query::{
    Alias, Asterisk, Expr, Func, Iden, LogicalChainOper, Order, PgFunc,
    PostgresQueryBuilder, Query, SimpleExpr,
};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::db::{build_simple_plain_ts_query, DbError};

#[derive(Iden)]
#[iden(rename = "market_booths")]
pub enum MarketBoothIden {
    Table,
    MarketBoothId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Name,
    NameTs,
    Description,
    DescriptionTs,
    ImageUrlPath,
}

#[derive(Debug, Clone)]
pub struct MarketBooth {
    pub market_booth_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: Option<String>,
    pub image_url_path: Option<String>,
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
            .into_table(MarketBoothIden::Table)
            .columns([
                MarketBoothIden::UserId,
                MarketBoothIden::Name,
                MarketBoothIden::Description,
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
            .from(MarketBoothIden::Table)
            .and_where(
                Expr::col(MarketBoothIden::MarketBoothId).eq(*market_booth_id),
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

            query.column(Asterisk).from(MarketBoothIden::Table);

            if let Some(user_id) = user_id {
                query.and_where(Expr::col(MarketBoothIden::UserId).eq(user_id));
            } else {
                query.order_by_expr(
                    SimpleExpr::FunctionCall(Func::random()),
                    Order::Asc,
                );
            }

            query
                .limit(limit)
                .offset(offset)
                .build_postgres(PostgresQueryBuilder)
        };

        let rows = client.query(sql.as_str(), &values.as_params()).await?;

        Ok(rows.iter().map(Self::from).collect())
    }

    pub async fn search(
        pool: &Pool,
        limit: u64,
        offset: u64,
        name_search: Option<String>,
        description_search: Option<String>,
    ) -> Result<Vec<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::select();
            query.column(Asterisk).from(MarketBoothIden::Table);

            if let Some(name_query) = name_search {
                let tsquery = build_simple_plain_ts_query(name_query);

                let rank_alias = Alias::new("name_rank");

                query
                    .expr_as(
                        Expr::expr(PgFunc::ts_rank(
                            Expr::col(MarketBoothIden::NameTs),
                            tsquery.clone(),
                        )),
                        rank_alias.clone(),
                    )
                    .and_or_where(LogicalChainOper::Or(
                        Expr::col(MarketBoothIden::NameTs).matches(tsquery),
                    ))
                    .order_by(rank_alias, Order::Desc);
            }

            if let Some(description_query) = description_search {
                let tsquery = build_simple_plain_ts_query(description_query);

                let rank_alias = Alias::new("description_rank");

                query
                    .expr_as(
                        Expr::expr(PgFunc::ts_rank(
                            Expr::col(MarketBoothIden::DescriptionTs),
                            tsquery.clone(),
                        )),
                        rank_alias.clone(),
                    )
                    .and_or_where(LogicalChainOper::Or(
                        Expr::col(MarketBoothIden::DescriptionTs)
                            .matches(tsquery),
                    ))
                    .order_by(rank_alias, Order::Desc);
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
        user_id: &String,
        market_booth_id: &Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Self, DbError> {
        let client = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::update();
            query.table(MarketBoothIden::Table);

            if let Some(name) = name {
                query.value(MarketBoothIden::Name, name);
            }

            if let Some(description) = description {
                query.value(MarketBoothIden::Description, description);
            }

            query
                .and_where(Expr::col(MarketBoothIden::UserId).eq(user_id))
                .and_where(
                    Expr::col(MarketBoothIden::MarketBoothId)
                        .eq(*market_booth_id),
                )
                .returning_all();

            query.build_postgres(PostgresQueryBuilder)
        };

        let row = client.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn delete(
        pool: &Pool,
        user_id: &String,
        market_booth_id: &Uuid,
    ) -> Result<Self, DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::delete()
            .from_table(MarketBoothIden::Table)
            .and_where(Expr::col(MarketBoothIden::UserId).eq(user_id))
            .and_where(
                Expr::col(MarketBoothIden::MarketBoothId).eq(*market_booth_id),
            )
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = client.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn update_image_url_path(
        pool: &Pool,
        user_id: &String,
        market_booth_id: &Uuid,
        image_url_path: Option<String>,
    ) -> Result<Self, DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::update()
            .table(MarketBoothIden::Table)
            .value(MarketBoothIden::ImageUrlPath, image_url_path)
            .and_where(Expr::col(MarketBoothIden::UserId).eq(user_id))
            .and_where(
                Expr::col(MarketBoothIden::MarketBoothId).eq(*market_booth_id),
            )
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = client.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
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
            image_url_path: row.get("image_url_path"),
        }
    }
}

impl From<Row> for MarketBooth {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}
