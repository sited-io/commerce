use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::Pool;
use sea_query::extension::postgres::PgExpr;
use sea_query::{
    Alias, Asterisk, Expr, Func, Iden, IntoColumnRef, LogicalChainOper, Order,
    PgFunc, PostgresQueryBuilder, Query, SelectStatement, SimpleExpr,
};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::api::peoplesmarkets::commerce::v1::{
    OffersFilterField, OffersOrderByField,
};
use crate::api::peoplesmarkets::ordering::v1::Direction;
use crate::db::{build_simple_plain_ts_query, DbError};

use super::offer_image::{OfferImageAsRel, OfferImageAsRelVec};
use super::offer_price::{OfferPriceAsRel, OfferPriceAsRelVec, OfferPriceIden};
use super::{MarketBoothIden, OfferImageIden};

#[derive(Debug, Clone, Iden)]
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
    IsActive,
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
    pub is_active: bool,
    pub images: Vec<OfferImageAsRel>,
    pub price: Option<OfferPriceAsRel>,
    pub market_booth_name: String,
}

impl Offer {
    const OFFER_IMAGES_ALIAS: &str = "images";
    const OFFER_PRICES_ALIAS: &str = "prices";
    const MARKET_BOOTH_NAME_ALIAS: &str = "market_booth_name";

    fn get_offer_images_alias() -> Alias {
        Alias::new(Self::OFFER_IMAGES_ALIAS)
    }

    fn get_offer_price_alias() -> Alias {
        Alias::new(Self::OFFER_PRICES_ALIAS)
    }

    fn get_market_booth_name_alias() -> Alias {
        Alias::new(Self::MARKET_BOOTH_NAME_ALIAS)
    }

    fn select_with_relations() -> SelectStatement {
        let mut query = Query::select();

        query
            .column((OfferIden::Table, Asterisk))
            .expr_as(OfferImageAsRel::get_agg(), Self::get_offer_images_alias())
            .expr_as(OfferPriceAsRel::get_agg(), Self::get_offer_price_alias())
            .expr_as(
                Expr::col((MarketBoothIden::Table, MarketBoothIden::Name)),
                Self::get_market_booth_name_alias(),
            )
            .from(OfferIden::Table)
            .left_join(
                OfferImageIden::Table,
                Expr::col((OfferIden::Table, OfferIden::OfferId))
                    .equals((OfferImageIden::Table, OfferImageIden::OfferId)),
            )
            .left_join(
                OfferPriceIden::Table,
                Expr::col((OfferIden::Table, OfferIden::OfferId))
                    .equals((OfferPriceIden::Table, OfferPriceIden::OfferId)),
            )
            .left_join(
                MarketBoothIden::Table,
                Expr::col((OfferIden::Table, OfferIden::MarketBoothId)).equals(
                    (MarketBoothIden::Table, MarketBoothIden::MarketBoothId),
                ),
            )
            .group_by_columns([
                (OfferIden::Table, OfferIden::OfferId).into_column_ref(),
                Self::get_market_booth_name_alias().into_column_ref(),
            ]);

        query
    }

    fn add_order_by(
        query: &mut SelectStatement,
        order_by_field: OffersOrderByField,
        order_by_direction: Direction,
    ) {
        use OffersOrderByField::*;

        let order = match order_by_direction {
            Direction::Unspecified | Direction::Asc => Order::Asc,
            Direction::Desc => Order::Desc,
        };

        match order_by_field {
            Unspecified | CreatedAt => {
                query.order_by((OfferIden::Table, OfferIden::CreatedAt), order)
            }
            UpdatedAt => {
                query.order_by((OfferIden::Table, OfferIden::UpdatedAt), order)
            }
            Name => query.order_by((OfferIden::Table, OfferIden::Name), order),
            Random => query.order_by_expr(
                SimpleExpr::FunctionCall(Func::random()),
                Order::Asc,
            ),
        };
    }

    fn add_filter(
        query: &mut SelectStatement,
        filter_field: OffersFilterField,
        filter_query: String,
    ) {
        use OffersFilterField::*;

        match filter_field {
            Unspecified => {}
            Name => Self::add_ts_filter(
                query,
                (OfferIden::Table, OfferIden::NameTs),
                &filter_query,
            ),
            Description => Self::add_ts_filter(
                query,
                (OfferIden::Table, OfferIden::DescriptionTs),
                &filter_query,
            ),
            NameAndDescription => {
                Self::add_ts_filter(
                    query,
                    (OfferIden::Table, OfferIden::NameTs),
                    &filter_query,
                );
                Self::add_ts_filter(
                    query,
                    (OfferIden::Table, OfferIden::DescriptionTs),
                    &filter_query,
                );
            }
        }
    }

    fn add_ts_filter(
        query: &mut SelectStatement,
        col: (OfferIden, OfferIden),
        filter_query: &String,
    ) {
        let tsquery = build_simple_plain_ts_query(filter_query);
        let rank_alias = Alias::new(format!("{}_rank", col.1.to_string()));
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

        let (sql, values) = Self::select_with_relations()
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
        filter: Option<(OffersFilterField, String)>,
        order_by: Option<(OffersOrderByField, Direction)>,
    ) -> Result<Vec<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = {
            let mut query = Self::select_with_relations();

            if let Some(market_booth_id) = market_booth_id {
                query.and_where(
                    Expr::col((OfferIden::Table, OfferIden::MarketBoothId))
                        .eq(market_booth_id),
                );
            }

            if let Some(user_id) = user_id {
                query.and_where(
                    Expr::col((OfferIden::Table, OfferIden::UserId))
                        .eq(user_id),
                );
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
        offer_id: &Uuid,
        name: Option<String>,
        description: Option<String>,
        is_active: Option<bool>,
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

            if let Some(is_active) = is_active {
                query.value(OfferIden::IsActive, is_active);
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
        let mut images = images.map(|i| i.0).unwrap_or_default();
        images.sort_by(|a, b| a.ordering.cmp(&b.ordering));

        let prices: Option<OfferPriceAsRelVec> =
            row.try_get(Self::OFFER_PRICES_ALIAS).ok();

        Self {
            offer_id: row.get(OfferIden::OfferId.to_string().as_str()),
            market_booth_id: row
                .get(OfferIden::MarketBoothId.to_string().as_str()),
            user_id: row.get(OfferIden::UserId.to_string().as_str()),
            created_at: row.get(OfferIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(OfferIden::UpdatedAt.to_string().as_str()),
            name: row.get(OfferIden::Name.to_string().as_str()),
            description: row.get(OfferIden::Description.to_string().as_str()),
            is_active: row.get(OfferIden::IsActive.to_string().as_str()),
            images,
            price: prices.and_then(|p| p.0.first().cloned()),
            market_booth_name: row
                .try_get(Self::MARKET_BOOTH_NAME_ALIAS)
                .unwrap_or_default(),
        }
    }
}

impl From<Row> for Offer {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}
