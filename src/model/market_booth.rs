use chrono::{DateTime, Utc};
use deadpool_postgres::{tokio_postgres::Row, Pool};
use sea_query::extension::postgres::PgExpr;
use sea_query::{
    Alias, Asterisk, Expr, Func, Iden, LogicalChainOper, Order, PgFunc,
    PostgresQueryBuilder, Query, SelectStatement, SimpleExpr,
};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::api::peoplesmarkets::commerce::v1::{
    MarketBoothsFilterField, MarketBoothsOrderByField,
};
use crate::api::peoplesmarkets::ordering::v1::Direction;
use crate::db::{build_simple_plain_ts_query, DbError};

#[derive(Debug, Clone, Iden)]
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
        filter: Option<(MarketBoothsFilterField, String)>,
        order_by: Option<(MarketBoothsOrderByField, Direction)>,
    ) -> Result<Vec<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::select();

            query.column(Asterisk).from(MarketBoothIden::Table);

            if let Some(user_id) = user_id {
                query.and_where(Expr::col(MarketBoothIden::UserId).eq(user_id));
            }

            if let Some((filter_field, filter_query)) = filter {
                Self::add_filter(&mut query, filter_field, filter_query);
            }

            if let Some((order_by_field, order_by_direction)) = order_by {
                Self::add_order_by(
                    &mut query,
                    order_by_field,
                    order_by_direction,
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

    //
    // private methods
    //
    fn add_order_by(
        query: &mut SelectStatement,
        order_by_field: MarketBoothsOrderByField,
        order_by_direction: Direction,
    ) {
        use MarketBoothsOrderByField::*;

        let order = match order_by_direction {
            Direction::Unspecified | Direction::Asc => Order::Asc,
            Direction::Desc => Order::Desc,
        };

        match order_by_field {
            Unspecified | CreatedAt => {
                query.order_by(MarketBoothIden::CreatedAt, order)
            }
            UpdatedAt => query.order_by(MarketBoothIden::UpdatedAt, order),
            Name => query.order_by(MarketBoothIden::Name, order),
            Random => query.order_by_expr(
                SimpleExpr::FunctionCall(Func::random()),
                Order::Asc,
            ),
        };
    }

    fn add_filter(
        query: &mut SelectStatement,
        filter_field: MarketBoothsFilterField,
        filter_query: String,
    ) {
        use MarketBoothsFilterField::*;

        match filter_field {
            Unspecified => {}
            Name => Self::add_ts_filter(
                query,
                MarketBoothIden::NameTs,
                &filter_query,
            ),
            Description => Self::add_ts_filter(
                query,
                MarketBoothIden::DescriptionTs,
                &filter_query,
            ),
            NameAndDescription => {
                Self::add_ts_filter(
                    query,
                    MarketBoothIden::NameTs,
                    &filter_query,
                );
                Self::add_ts_filter(
                    query,
                    MarketBoothIden::DescriptionTs,
                    &filter_query,
                );
            }
        }
    }

    fn add_ts_filter(
        query: &mut SelectStatement,
        col: MarketBoothIden,
        filter_query: &String,
    ) {
        let tsquery = build_simple_plain_ts_query(filter_query);
        let rank_alias = Alias::new(format!("{}_rank", col.to_string()));
        query
            .expr_as(
                Expr::expr(PgFunc::ts_rank(
                    Expr::col(col.clone()),
                    tsquery.clone(),
                )),
                rank_alias.clone(),
            )
            .and_or_where(LogicalChainOper::Or(Expr::col(col).matches(tsquery)))
            .order_by(rank_alias, Order::Desc);
    }
}

impl From<&Row> for MarketBooth {
    fn from(row: &Row) -> Self {
        Self {
            market_booth_id: row
                .get(MarketBoothIden::MarketBoothId.to_string().as_str()),
            user_id: row.get(MarketBoothIden::UserId.to_string().as_str()),
            created_at: row
                .get(MarketBoothIden::CreatedAt.to_string().as_str()),
            updated_at: row
                .get(MarketBoothIden::UpdatedAt.to_string().as_str()),
            name: row.get(MarketBoothIden::Name.to_string().as_str()),
            description: row
                .get(MarketBoothIden::Description.to_string().as_str()),
            image_url_path: row
                .get(MarketBoothIden::ImageUrlPath.to_string().as_str()),
        }
    }
}

impl From<Row> for MarketBooth {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}
