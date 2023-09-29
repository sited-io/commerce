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
    ShopsFilterField, ShopsOrderByField,
};
use crate::api::peoplesmarkets::ordering::v1::Direction;
use crate::db::{build_simple_plain_ts_query, DbError};

use super::shop_customization::{
    ShopCustomizationAsRel, ShopCustomizationAsRelVec, ShopCustomizationIden,
};

#[derive(Debug, Clone, Iden)]
#[iden(rename = "shops")]
pub enum ShopIden {
    Table,
    ShopId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Slug,
    Domain,
    Name,
    NameTs,
    Description,
    DescriptionTs,
    PlatformFeePercent,
    MinimumPlatformFeeCent,
}

#[derive(Debug, Clone)]
pub struct Shop {
    pub shop_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub slug: String,
    pub domain: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub platform_fee_percent: u32,
    pub minimum_platform_fee_cent: u32,
    pub customization: Option<ShopCustomizationAsRel>,
}

impl Shop {
    const SHOP_CUSTOMIZATION_ALIAS: &str = "shop_customization";

    fn get_shop_customization_alias() -> Alias {
        Alias::new(Self::SHOP_CUSTOMIZATION_ALIAS)
    }

    fn select_with_relations() -> SelectStatement {
        let mut query = Query::select();

        query
            .column((ShopIden::Table, Asterisk))
            .expr_as(
                ShopCustomizationAsRel::get_agg(),
                Self::get_shop_customization_alias(),
            )
            .from(ShopIden::Table)
            .left_join(
                ShopCustomizationIden::Table,
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::ShopId,
                ))
                .equals((ShopIden::Table, ShopIden::ShopId)),
            )
            .group_by_col((ShopIden::Table, ShopIden::ShopId));

        query
    }

    fn add_order_by(
        query: &mut SelectStatement,
        order_by_field: ShopsOrderByField,
        order_by_direction: Direction,
    ) {
        use ShopsOrderByField::*;

        let order = match order_by_direction {
            Direction::Unspecified | Direction::Asc => Order::Asc,
            Direction::Desc => Order::Desc,
        };

        match order_by_field {
            Unspecified | CreatedAt => {
                query.order_by((ShopIden::Table, ShopIden::CreatedAt), order)
            }
            UpdatedAt => {
                query.order_by((ShopIden::Table, ShopIden::UpdatedAt), order)
            }
            Name => query.order_by((ShopIden::Table, ShopIden::Name), order),
            Random => query.order_by_expr(
                SimpleExpr::FunctionCall(Func::random()),
                Order::Asc,
            ),
        };
    }

    fn add_filter(
        query: &mut SelectStatement,
        filter_field: ShopsFilterField,
        filter_query: String,
    ) {
        use ShopsFilterField::*;

        match filter_field {
            Unspecified => {}
            Name => Self::add_ts_filter(query, ShopIden::NameTs, &filter_query),
            Description => Self::add_ts_filter(
                query,
                ShopIden::DescriptionTs,
                &filter_query,
            ),
            NameAndDescription => {
                Self::add_ts_filter(query, ShopIden::NameTs, &filter_query);
                Self::add_ts_filter(
                    query,
                    ShopIden::DescriptionTs,
                    &filter_query,
                );
            }
        }
    }

    fn add_ts_filter(
        query: &mut SelectStatement,
        col: ShopIden,
        filter_query: &String,
    ) {
        let tsquery = build_simple_plain_ts_query(filter_query);
        let rank_alias = Alias::new(format!("{}_rank", col.to_string()));
        query
            .expr_as(
                Expr::expr(PgFunc::ts_rank(
                    Expr::col((ShopIden::Table, col.clone())),
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
            .into_table(ShopIden::Table)
            .columns([
                ShopIden::UserId,
                ShopIden::Name,
                ShopIden::Slug,
                ShopIden::Description,
                ShopIden::PlatformFeePercent,
                ShopIden::MinimumPlatformFeeCent,
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
        shop_id: &Uuid,
        extended: bool,
    ) -> Result<Option<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = if extended {
            Self::select_with_relations()
        } else {
            Query::select()
                .column((ShopIden::Table, Asterisk))
                .from(ShopIden::Table)
                .to_owned()
        }
        .and_where(Expr::col((ShopIden::Table, ShopIden::ShopId)).eq(*shop_id))
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
            .from(ShopIden::Table)
            .and_where(Expr::col(ShopIden::Slug).eq(slug))
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
            .from(ShopIden::Table)
            .and_where(Expr::col(ShopIden::Domain).eq(domain))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn list(
        pool: &Pool,
        user_id: Option<&String>,
        limit: u64,
        offset: u64,
        filter: Option<(ShopsFilterField, String)>,
        order_by: Option<(ShopsOrderByField, Direction)>,
        extended: bool,
    ) -> Result<Vec<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = {
            let mut query = if extended {
                Self::select_with_relations()
            } else {
                Query::select()
                    .column((ShopIden::Table, Asterisk))
                    .from(ShopIden::Table)
                    .to_owned()
            };

            if let Some(user_id) = user_id {
                query.and_where(
                    Expr::col((ShopIden::Table, ShopIden::UserId)).eq(user_id),
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

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        pool: &Pool,
        user_id: &String,
        shop_id: &Uuid,
        name: Option<String>,
        slug: Option<String>,
        description: Option<String>,
        platform_fee_percent: Option<u32>,
        minimum_platform_fee_cent: Option<u32>,
    ) -> Result<Self, DbError> {
        let client = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::update();
            query.table(ShopIden::Table);

            if let Some(name) = name {
                query.value(ShopIden::Name, name);
            }

            if let Some(slug) = slug {
                query.value(ShopIden::Slug, slug);
            }

            if let Some(description) = description {
                query.value(ShopIden::Description, description);
            }

            if let Some(pfp) = platform_fee_percent {
                query.value(ShopIden::PlatformFeePercent, i64::from(pfp));
            }

            if let Some(mpfc) = minimum_platform_fee_cent {
                query.value(ShopIden::MinimumPlatformFeeCent, i64::from(mpfc));
            }

            query
                .and_where(Expr::col(ShopIden::UserId).eq(user_id))
                .and_where(Expr::col(ShopIden::ShopId).eq(*shop_id))
                .returning_all();

            query.build_postgres(PostgresQueryBuilder)
        };

        let row = client.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn update_domain<'a>(
        transaction: &Transaction<'a>,
        shop_id: &Uuid,
        domain: Option<String>,
    ) -> Result<Self, DbError> {
        let (sql, values) = Query::update()
            .table(ShopIden::Table)
            .value(ShopIden::Domain, domain)
            .and_where(Expr::col(ShopIden::ShopId).eq(*shop_id))
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
    ) -> Result<Self, DbError> {
        let (sql, values) = Query::delete()
            .from_table(ShopIden::Table)
            .and_where(Expr::col(ShopIden::UserId).eq(user_id))
            .and_where(Expr::col(ShopIden::ShopId).eq(*shop_id))
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = transaction
            .query_one(sql.as_str(), &values.as_params())
            .await?;

        Ok(Self::from(row))
    }
}

impl From<&Row> for Shop {
    fn from(row: &Row) -> Self {
        let customization: Option<ShopCustomizationAsRelVec> =
            row.try_get(Self::SHOP_CUSTOMIZATION_ALIAS).ok();

        Self {
            shop_id: row.get(ShopIden::ShopId.to_string().as_str()),
            user_id: row.get(ShopIden::UserId.to_string().as_str()),
            created_at: row.get(ShopIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(ShopIden::UpdatedAt.to_string().as_str()),
            name: row.get(ShopIden::Name.to_string().as_str()),
            slug: row.get(ShopIden::Slug.to_string().as_str()),
            description: row.get(ShopIden::Description.to_string().as_str()),
            platform_fee_percent: u32::try_from(row.get::<&str, i64>(
                ShopIden::PlatformFeePercent.to_string().as_str(),
            ))
            .expect("Should never be greater than 100"),
            minimum_platform_fee_cent: u32::try_from(row.get::<&str, i64>(
                ShopIden::MinimumPlatformFeeCent.to_string().as_str(),
            ))
            .expect("Should not be greater than 4294967295"),
            customization: customization.and_then(|c| c.0.first().cloned()),
            domain: row.get(ShopIden::Domain.to_string().as_str()),
        }
    }
}

impl From<Row> for Shop {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}
