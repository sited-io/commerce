use chrono::{DateTime, Utc};
use deadpool_postgres::Transaction;
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

use super::shop_customization::{
    ShopCustomizationAsRel, ShopCustomizationAsRelVec, ShopCustomizationIden,
};

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
    Slug,
    Description,
    DescriptionTs,
    ImageUrlPath,
    PlatformFeePercent,
    MinimumPlatformFeeCent,
    Domain,
}

#[derive(Debug, Clone)]
pub struct MarketBooth {
    pub market_booth_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub image_url_path: Option<String>,
    pub platform_fee_percent: u32,
    pub minimum_platform_fee_cent: u32,
    pub customization: Option<ShopCustomizationAsRel>,
    pub domain: Option<String>,
}

impl MarketBooth {
    const SHOP_CUSTOMIZATION_ALIAS: &str = "shop_customization";

    fn get_shop_customization_alias() -> Alias {
        Alias::new(Self::SHOP_CUSTOMIZATION_ALIAS)
    }

    fn select_with_relations() -> SelectStatement {
        let mut query = Query::select();

        query
            .column((MarketBoothIden::Table, Asterisk))
            .expr_as(
                ShopCustomizationAsRel::get_agg(),
                Self::get_shop_customization_alias(),
            )
            .from(MarketBoothIden::Table)
            .left_join(
                ShopCustomizationIden::Table,
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::ShopId,
                ))
                .equals((
                    MarketBoothIden::Table,
                    MarketBoothIden::MarketBoothId,
                )),
            )
            .group_by_col((
                MarketBoothIden::Table,
                MarketBoothIden::MarketBoothId,
            ));

        query
    }

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
            Unspecified | CreatedAt => query.order_by(
                (MarketBoothIden::Table, MarketBoothIden::CreatedAt),
                order,
            ),
            UpdatedAt => query.order_by(
                (MarketBoothIden::Table, MarketBoothIden::UpdatedAt),
                order,
            ),
            Name => query.order_by(
                (MarketBoothIden::Table, MarketBoothIden::Name),
                order,
            ),
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
                    Expr::col((MarketBoothIden::Table, col.clone())),
                    tsquery.clone(),
                )),
                rank_alias.clone(),
            )
            .and_or_where(LogicalChainOper::Or(Expr::col(col).matches(tsquery)))
            .order_by(rank_alias, Order::Desc);
    }

    pub async fn create(
        pool: &Pool,
        user_id: &String,
        name: &String,
        slug: &String,
        description: Option<String>,
        platform_fee_percent: u32,
        minimum_platform_fee_cent: u32,
    ) -> Result<Self, DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(MarketBoothIden::Table)
            .columns([
                MarketBoothIden::UserId,
                MarketBoothIden::Name,
                MarketBoothIden::Slug,
                MarketBoothIden::Description,
                MarketBoothIden::PlatformFeePercent,
                MarketBoothIden::MinimumPlatformFeeCent,
            ])
            .values([
                user_id.into(),
                name.into(),
                slug.into(),
                description.unwrap_or_default().into(),
                i64::from(platform_fee_percent).into(),
                i64::from(minimum_platform_fee_cent).into(),
            ])?
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = client.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn get(
        pool: &Pool,
        market_booth_id: &Uuid,
        extended: bool,
    ) -> Result<Option<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = if extended {
            Self::select_with_relations()
        } else {
            Query::select()
                .column((MarketBoothIden::Table, Asterisk))
                .from(MarketBoothIden::Table)
                .to_owned()
        }
        .and_where(
            Expr::col((MarketBoothIden::Table, MarketBoothIden::MarketBoothId))
                .eq(*market_booth_id),
        )
        .build_postgres(PostgresQueryBuilder);

        Ok(client
            .query_opt(sql.as_str(), &values.as_params())
            .await?
            .map(Self::from))
    }

    pub async fn get_by_slug(
        pool: &Pool,
        slug: &String,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(MarketBoothIden::Table)
            .and_where(Expr::col(MarketBoothIden::Slug).eq(slug))
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
            .from(MarketBoothIden::Table)
            .and_where(Expr::col(MarketBoothIden::Domain).eq(domain))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn list(
        pool: &Pool,
        user_id: Option<&String>,
        limit: u64,
        offset: u64,
        filter: Option<(MarketBoothsFilterField, String)>,
        order_by: Option<(MarketBoothsOrderByField, Direction)>,
        extended: bool,
    ) -> Result<Vec<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = {
            let mut query = if extended {
                Self::select_with_relations()
            } else {
                Query::select()
                    .column((MarketBoothIden::Table, Asterisk))
                    .from(MarketBoothIden::Table)
                    .to_owned()
            };

            if let Some(user_id) = user_id {
                query.and_where(
                    Expr::col((
                        MarketBoothIden::Table,
                        MarketBoothIden::UserId,
                    ))
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
        market_booth_id: &Uuid,
        name: Option<String>,
        slug: Option<String>,
        description: Option<String>,
        platform_fee_percent: Option<u32>,
        minimum_platform_fee_cent: Option<u32>,
    ) -> Result<Self, DbError> {
        let client = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::update();
            query.table(MarketBoothIden::Table);

            if let Some(name) = name {
                query.value(MarketBoothIden::Name, name);
            }

            if let Some(slug) = slug {
                query.value(MarketBoothIden::Slug, slug);
            }

            if let Some(description) = description {
                query.value(MarketBoothIden::Description, description);
            }

            if let Some(pfp) = platform_fee_percent {
                query
                    .value(MarketBoothIden::PlatformFeePercent, i64::from(pfp));
            }

            if let Some(mpfc) = minimum_platform_fee_cent {
                query.value(
                    MarketBoothIden::MinimumPlatformFeeCent,
                    i64::from(mpfc),
                );
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

    pub async fn delete<'a>(
        transaction: &Transaction<'a>,
        user_id: &String,
        market_booth_id: &Uuid,
    ) -> Result<Self, DbError> {
        let (sql, values) = Query::delete()
            .from_table(MarketBoothIden::Table)
            .and_where(Expr::col(MarketBoothIden::UserId).eq(user_id))
            .and_where(
                Expr::col(MarketBoothIden::MarketBoothId).eq(*market_booth_id),
            )
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = transaction
            .query_one(sql.as_str(), &values.as_params())
            .await?;

        Ok(Self::from(row))
    }
}

impl From<&Row> for MarketBooth {
    fn from(row: &Row) -> Self {
        let customization: Option<ShopCustomizationAsRelVec> =
            row.try_get(Self::SHOP_CUSTOMIZATION_ALIAS).ok();

        Self {
            market_booth_id: row
                .get(MarketBoothIden::MarketBoothId.to_string().as_str()),
            user_id: row.get(MarketBoothIden::UserId.to_string().as_str()),
            created_at: row
                .get(MarketBoothIden::CreatedAt.to_string().as_str()),
            updated_at: row
                .get(MarketBoothIden::UpdatedAt.to_string().as_str()),
            name: row.get(MarketBoothIden::Name.to_string().as_str()),
            slug: row.get(MarketBoothIden::Slug.to_string().as_str()),
            description: row
                .get(MarketBoothIden::Description.to_string().as_str()),
            image_url_path: row
                .get(MarketBoothIden::ImageUrlPath.to_string().as_str()),
            platform_fee_percent: u32::try_from(row.get::<&str, i64>(
                MarketBoothIden::PlatformFeePercent.to_string().as_str(),
            ))
            .expect("Should never be greater than 100"),
            minimum_platform_fee_cent: u32::try_from(row.get::<&str, i64>(
                MarketBoothIden::MinimumPlatformFeeCent.to_string().as_str(),
            ))
            .expect("Should not be greater than 4294967295"),
            customization: customization.and_then(|c| c.0.first().cloned()),
            domain: row.get(MarketBoothIden::Domain.to_string().as_str()),
        }
    }
}

impl From<Row> for MarketBooth {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}
