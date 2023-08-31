use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::Pool;
use sea_query::extension::postgres::PgExpr;
use sea_query::{
    Alias, Asterisk, Expr, Func, Iden, LogicalChainOper, Order, PgFunc,
    PostgresQueryBuilder, Query, SelectStatement, SimpleExpr,
};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::db::{build_simple_plain_ts_query, ArrayAgg, DbError};

use super::offer_image::{OfferImageAsRel, OfferImageAsRelVec};
use super::OfferImageIden;

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
    NameTs,
    Description,
    DescriptionTs,
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
    pub images: Vec<OfferImageAsRel>,
}

impl Offer {
    const OFFER_IMAGES_ALIAS: &str = "images";

    fn get_offer_images_alias() -> Alias {
        Alias::new(Self::OFFER_IMAGES_ALIAS)
    }

    fn get_offer_image_agg() -> SimpleExpr {
        Func::cust(ArrayAgg)
            .args([Expr::tuple([
                Expr::col((
                    OfferImageIden::Table,
                    OfferImageIden::OfferImageId,
                ))
                .into(),
                Expr::col((
                    OfferImageIden::Table,
                    OfferImageIden::ImageUrlPath,
                ))
                .into(),
                Expr::col((OfferImageIden::Table, OfferImageIden::Ordering))
                    .into(),
            ])
            .into()])
            .into()
    }

    fn get_select_with_offer_images() -> SelectStatement {
        let mut query = Query::select();

        query
            .column((OfferIden::Table, Asterisk))
            .expr_as(
                Self::get_offer_image_agg(),
                Self::get_offer_images_alias(),
            )
            .from(OfferIden::Table)
            .left_join(
                OfferImageIden::Table,
                Expr::col((OfferIden::Table, OfferIden::OfferId))
                    .equals((OfferImageIden::Table, OfferImageIden::OfferId)),
            )
            .group_by_col((OfferIden::Table, OfferIden::OfferId));

        query
    }

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

        let (sql, values) = Self::get_select_with_offer_images()
            .and_where(
                Expr::col((OfferIden::Table, OfferIden::OfferId)).eq(*offer_id),
            )
            .build_postgres(PostgresQueryBuilder);

        let row = client.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
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
            let mut query = Self::get_select_with_offer_images();

            if let Some(market_booth_id) = market_booth_id {
                query.and_where(
                    Expr::col(OfferIden::MarketBoothId).eq(market_booth_id),
                );
            }

            if let Some(user_id) = user_id {
                query.and_where(
                    Expr::col((OfferIden::Table, OfferIden::UserId))
                        .eq(user_id),
                );
            }

            if market_booth_id.is_none() && user_id.is_none() {
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
            let mut query = Self::get_select_with_offer_images();

            if let Some(name_query) = name_search {
                let tsquery = build_simple_plain_ts_query(name_query);

                let rank_alias = Alias::new("name_rank");

                query
                    .expr_as(
                        Expr::expr(PgFunc::ts_rank(
                            Expr::col(OfferIden::NameTs),
                            tsquery.clone(),
                        )),
                        rank_alias.clone(),
                    )
                    .and_or_where(LogicalChainOper::Or(
                        Expr::col(OfferIden::NameTs).matches(tsquery),
                    ))
                    .order_by(rank_alias, Order::Desc);
            }

            if let Some(description_query) = description_search {
                let tsquery = build_simple_plain_ts_query(description_query);

                let rank_alias = Alias::new("description_rank");

                query
                    .expr_as(
                        Expr::expr(PgFunc::ts_rank(
                            Expr::col(OfferIden::DescriptionTs),
                            tsquery.clone(),
                        )),
                        rank_alias.clone(),
                    )
                    .and_or_where(LogicalChainOper::Or(
                        Expr::col(OfferIden::DescriptionTs).matches(tsquery),
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
                .and_where(Expr::col(OfferIden::UserId).eq(user_id))
                .and_where(Expr::col(OfferIden::OfferId).eq(*offer_id))
                .returning_all();

            query.build_postgres(PostgresQueryBuilder)
        };

        let row = client.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn delete(
        pool: &Pool,
        user_id: &String,
        offer_id: &Uuid,
    ) -> Result<(), DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::delete()
            .from_table(OfferIden::Table)
            .and_where(Expr::col(OfferIden::UserId).eq(user_id))
            .and_where(Expr::col(OfferIden::OfferId).eq(*offer_id))
            .build_postgres(PostgresQueryBuilder);

        client.execute(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }
}

impl From<&Row> for Offer {
    fn from(row: &Row) -> Self {
        let images: Option<OfferImageAsRelVec> =
            row.try_get(Self::OFFER_IMAGES_ALIAS).ok();
        let images = images.map(|i| i.0).unwrap_or_default();

        Self {
            offer_id: row.get("offer_id"),
            market_booth_id: row.get("market_booth_id"),
            user_id: row.get("user_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            name: row.get("name"),
            description: row.get("description"),
            images,
        }
    }
}

impl From<Row> for Offer {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}
