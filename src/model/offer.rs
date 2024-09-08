use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::{Pool, Transaction};
use sea_query::extension::postgres::PgExpr;
use sea_query::{
    all, any, Alias, Asterisk, Expr, Func, Iden, IntoColumnRef, Order, PgFunc,
    PostgresQueryBuilder, Query, SelectStatement, SimpleExpr,
};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::api::sited_io::commerce::v1::{
    OffersFilterField, OffersOrderByField,
};
use crate::api::sited_io::types::v1::Direction;
use crate::db::{build_simple_plain_ts_query, get_count_from_rows, DbError};

use super::offer_image::{OfferImageAsRel, OfferImageAsRelVec};
use super::offer_price::{OfferPriceAsRel, OfferPriceAsRelVec, OfferPriceIden};
use super::{OfferImageIden, ShopIden};

#[derive(Debug, Clone, Copy, Iden)]
#[iden(rename = "offers")]
pub enum OfferIden {
    Table,
    OfferId,
    ShopId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Name,
    NameTs,
    Description,
    DescriptionTs,
    #[iden(rename = "type_")]
    Type,
    IsActive,
    IsFeatured,
}

#[derive(Debug, Clone)]
pub struct Offer {
    pub offer_id: Uuid,
    pub shop_id: Uuid,
    pub shop_name: String,
    pub shop_slug: String,
    pub shop_domain: Option<String>,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub is_featured: bool,
    pub type_: Option<String>,
    pub images: Vec<OfferImageAsRel>,
    pub price: Option<OfferPriceAsRel>,
}

impl Offer {
    const OFFER_IMAGES_ALIAS: &'static str = "images";
    const OFFER_PRICES_ALIAS: &'static str = "prices";
    const SHOP_NAME_ALIAS: &'static str = "shop_name";
    const SHOP_SLUG_ALIAS: &'static str = "shop_slug";
    const SHOP_DOMAIN_ALIAS: &'static str = "shop_domain";
    const NAME_TS_RANK_ALIAS: &'static str = "name_ts_rank";
    const DESCRIPTION_TS_RANK_ALIAS: &'static str = "description_ts_rank";

    fn get_offer_images_alias() -> Alias {
        Alias::new(Self::OFFER_IMAGES_ALIAS)
    }

    fn get_offer_price_alias() -> Alias {
        Alias::new(Self::OFFER_PRICES_ALIAS)
    }

    fn get_shop_name_alias() -> Alias {
        Alias::new(Self::SHOP_NAME_ALIAS)
    }

    fn get_shop_slug_alias() -> Alias {
        Alias::new(Self::SHOP_SLUG_ALIAS)
    }

    fn get_shop_domain_alias() -> Alias {
        Alias::new(Self::SHOP_DOMAIN_ALIAS)
    }

    fn get_name_ts_rank_alias() -> Alias {
        Alias::new(Self::NAME_TS_RANK_ALIAS)
    }

    fn get_description_ts_rank_alias() -> Alias {
        Alias::new(Self::DESCRIPTION_TS_RANK_ALIAS)
    }

    fn select_with_relations() -> SelectStatement {
        let mut query = Query::select();

        query
            .column((OfferIden::Table, Asterisk))
            .expr_as(OfferImageAsRel::get_agg(), Self::get_offer_images_alias())
            .expr_as(OfferPriceAsRel::get_agg(), Self::get_offer_price_alias())
            .expr_as(
                Expr::col((ShopIden::Table, ShopIden::Name)),
                Self::get_shop_name_alias(),
            )
            .expr_as(
                Expr::col((ShopIden::Table, ShopIden::Slug)),
                Self::get_shop_slug_alias(),
            )
            .expr_as(
                Expr::col((ShopIden::Table, ShopIden::Domain)),
                Self::get_shop_domain_alias(),
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
                ShopIden::Table,
                Expr::col((OfferIden::Table, OfferIden::ShopId))
                    .equals((ShopIden::Table, ShopIden::ShopId)),
            )
            .group_by_columns([
                (OfferIden::Table, OfferIden::OfferId).into_column_ref(),
                Self::get_shop_name_alias().into_column_ref(),
                Self::get_shop_slug_alias().into_column_ref(),
                Self::get_shop_domain_alias().into_column_ref(),
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
    ) -> Result<(), DbError> {
        use OffersFilterField::*;

        match filter_field {
            Unspecified => {}
            Name => {
                let column = (OfferIden::Table, OfferIden::NameTs);
                let tsquery = build_simple_plain_ts_query(&filter_query);
                query
                    .expr_as(
                        Expr::expr(PgFunc::ts_rank(
                            Expr::col(column),
                            tsquery.clone(),
                        )),
                        Self::get_name_ts_rank_alias(),
                    )
                    .cond_where(Expr::col(column).matches(tsquery));
            }
            Description => {
                let column = (OfferIden::Table, OfferIden::DescriptionTs);
                let tsquery = build_simple_plain_ts_query(&filter_query);
                query
                    .expr_as(
                        Expr::expr(PgFunc::ts_rank(
                            Expr::col(column),
                            tsquery.clone(),
                        )),
                        Self::get_description_ts_rank_alias(),
                    )
                    .cond_where(Expr::col(column).matches(tsquery));
            }
            NameAndDescription => {
                let name_col = (OfferIden::Table, OfferIden::NameTs);
                let description_col =
                    (OfferIden::Table, OfferIden::DescriptionTs);

                let tsquery = build_simple_plain_ts_query(&filter_query);

                query
                    .expr_as(
                        Expr::expr(PgFunc::ts_rank(
                            Expr::col(name_col),
                            tsquery.clone(),
                        )),
                        Self::get_name_ts_rank_alias(),
                    )
                    .expr_as(
                        Expr::expr(PgFunc::ts_rank(
                            Expr::col(description_col),
                            tsquery.clone(),
                        )),
                        Self::get_description_ts_rank_alias(),
                    )
                    .cond_where(any![
                        Expr::col(name_col).matches(tsquery.clone()),
                        Expr::col(description_col).matches(tsquery),
                    ]);
            }
            Type => {
                query.cond_where(
                    Expr::col((OfferIden::Table, OfferIden::Type))
                        .eq(filter_query),
                );
            }
            IsFeatured => {
                let filter_query: bool = filter_query
                    .trim()
                    .parse()
                    .map_err(|_| DbError::Argument("filter.query"))?;
                query.cond_where(
                    Expr::col((OfferIden::Table, OfferIden::IsFeatured))
                        .eq(filter_query),
                );
            }
        }

        Ok(())
    }

    pub async fn create(
        pool: &Pool,
        shop_id: Uuid,
        user_id: &String,
        name: String,
        description: Option<String>,
        type_: &str,
        is_featured: bool,
    ) -> Result<Self, DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(OfferIden::Table)
            .columns([
                OfferIden::ShopId,
                OfferIden::UserId,
                OfferIden::Name,
                OfferIden::Description,
                OfferIden::Type,
                OfferIden::IsFeatured,
            ])
            .values([
                shop_id.into(),
                user_id.into(),
                name.into(),
                description.unwrap_or_default().into(),
                type_.into(),
                is_featured.into(),
            ])?
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = client.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn get(
        pool: &Pool,
        offer_id: &Uuid,
        user_id: Option<&String>,
    ) -> Result<Option<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = Self::select_with_relations()
            .cond_where(all![
                Expr::col((OfferIden::Table, OfferIden::OfferId)).eq(*offer_id),
                any![
                    Expr::col((OfferIden::Table, OfferIden::IsActive)).eq(true),
                    Expr::col((OfferIden::Table, OfferIden::UserId))
                        .eq(user_id.cloned())
                ]
            ])
            .build_postgres(PostgresQueryBuilder);

        let row = client.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn get_for_user(
        pool: &Pool,
        user_id: &String,
        offer_id: &Uuid,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Self::select_with_relations()
            .and_where(
                Expr::col((OfferIden::Table, OfferIden::OfferId)).eq(*offer_id),
            )
            .and_where(
                Expr::col((OfferIden::Table, OfferIden::UserId)).eq(user_id),
            )
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list(
        pool: &Pool,
        shop_id: Option<Uuid>,
        user_id: Option<&String>,
        limit: u64,
        offset: u64,
        filter: Option<(OffersFilterField, String)>,
        order_by: Option<(OffersOrderByField, Direction)>,
        request_user_id: Option<&String>,
    ) -> Result<(Vec<Self>, i64), DbError> {
        let mut conn = pool.get().await?;
        let transaction = conn.transaction().await?;

        let (sql, values) = {
            let mut query = Self::select_with_relations();

            if let Some(shop_id) = shop_id {
                query.cond_where(
                    Expr::col((OfferIden::Table, OfferIden::ShopId))
                        .eq(shop_id),
                );
            }

            if let Some(user_id) = user_id {
                query.cond_where(
                    Expr::col((OfferIden::Table, OfferIden::UserId))
                        .eq(user_id),
                );
            }

            query.cond_where(any![
                Expr::col((OfferIden::Table, OfferIden::IsActive)).eq(true),
                Expr::col((OfferIden::Table, OfferIden::UserId))
                    .eq(request_user_id.cloned())
            ]);

            if let Some((filter_field, filter_query)) = filter.clone() {
                Self::add_filter(&mut query, filter_field, filter_query)?;
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

        let (count_sql, count_values) = {
            let mut query = Query::select();

            query
                .expr(Expr::col((OfferIden::Table, Asterisk)).count())
                .from(OfferIden::Table)
                .group_by_col((OfferIden::Table, OfferIden::OfferId));

            if let Some(shop_id) = shop_id {
                query.cond_where(
                    Expr::col((OfferIden::Table, OfferIden::ShopId))
                        .eq(shop_id),
                );
            }

            if let Some(user_id) = user_id {
                query.cond_where(
                    Expr::col((OfferIden::Table, OfferIden::UserId))
                        .eq(user_id),
                );
            }

            query.cond_where(any![
                Expr::col((OfferIden::Table, OfferIden::IsActive)).eq(true),
                Expr::col((OfferIden::Table, OfferIden::UserId))
                    .eq(request_user_id.cloned())
            ]);

            if let Some((filter_field, filter_query)) = filter {
                Self::add_filter(&mut query, filter_field, filter_query)?;
            }

            query.build_postgres(PostgresQueryBuilder)
        };

        let rows = transaction.query(sql.as_str(), &values.as_params()).await?;

        let count_rows = transaction
            .query(count_sql.as_str(), &count_values.as_params())
            .await?;

        let count = get_count_from_rows(&count_rows);

        transaction.commit().await?;

        Ok((rows.iter().map(Self::from).collect(), count))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        pool: &Pool,
        user_id: &String,
        offer_id: &Uuid,
        name: Option<String>,
        description: Option<String>,
        is_active: Option<bool>,
        type_: Option<&str>,
        is_featured: Option<bool>,
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

            if let Some(type_) = type_ {
                query.value(OfferIden::Type, type_);
            }

            if let Some(is_featured) = is_featured {
                query.value(OfferIden::IsFeatured, is_featured);
            }

            query
                .and_where(Expr::col(OfferIden::UserId).eq(user_id))
                .and_where(Expr::col(OfferIden::OfferId).eq(*offer_id))
                .returning_all()
                .build_postgres(PostgresQueryBuilder)
        };

        let row = client.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn deactivate_for_shop(
        pool: &Pool,
        user_id: &String,
        shop_id: &Uuid,
    ) -> Result<(), DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::update()
            .table(OfferIden::Table)
            .value(OfferIden::IsActive, false)
            .and_where(Expr::col(OfferIden::UserId).eq(user_id))
            .and_where(Expr::col(OfferIden::ShopId).eq(*shop_id))
            .build_postgres(PostgresQueryBuilder);

        conn.execute(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }

    pub async fn delete<'a>(
        transaction: &Transaction<'a>,
        user_id: &String,
        offer_id: &Uuid,
    ) -> Result<(), DbError> {
        let (sql, values) = Query::delete()
            .from_table(OfferIden::Table)
            .and_where(Expr::col(OfferIden::UserId).eq(user_id))
            .and_where(Expr::col(OfferIden::OfferId).eq(*offer_id))
            .build_postgres(PostgresQueryBuilder);

        transaction
            .execute(sql.as_str(), &values.as_params())
            .await?;

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
            shop_id: row.get(OfferIden::ShopId.to_string().as_str()),
            user_id: row.get(OfferIden::UserId.to_string().as_str()),
            created_at: row.get(OfferIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(OfferIden::UpdatedAt.to_string().as_str()),
            name: row.get(OfferIden::Name.to_string().as_str()),
            description: row.get(OfferIden::Description.to_string().as_str()),
            is_active: row.get(OfferIden::IsActive.to_string().as_str()),
            images,
            price: prices.and_then(|p| p.0.first().cloned()),
            shop_name: row.try_get(Self::SHOP_NAME_ALIAS).unwrap_or_default(),
            type_: row.get(OfferIden::Type.to_string().as_str()),
            is_featured: row.get(OfferIden::IsFeatured.to_string().as_str()),
            shop_slug: row.try_get(Self::SHOP_SLUG_ALIAS).unwrap_or_default(),
            shop_domain: row
                .try_get(Self::SHOP_DOMAIN_ALIAS)
                .unwrap_or_default(),
        }
    }
}

impl From<Row> for Offer {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}
