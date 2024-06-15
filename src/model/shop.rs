use chrono::{DateTime, Utc};
use deadpool_postgres::Transaction;
use deadpool_postgres::{tokio_postgres::Row, Pool};
use sea_query::extension::postgres::PgExpr;
use sea_query::{
    all, any, Alias, Asterisk, Expr, Func, Iden, Order, PgFunc,
    PostgresQueryBuilder, Query, SelectStatement, SimpleExpr,
};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::api::sited_io::commerce::v1::{
    ShopsFilterField, ShopsOrderByField,
};
use crate::api::sited_io::ordering::v1::Direction;
use crate::db::{build_simple_plain_ts_query, DbError};

use super::shop_customization::{
    ShopCustomizationAsRel, ShopCustomizationAsRelVec, ShopCustomizationIden,
};

#[derive(Debug, Clone, Copy, Iden)]
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
    IsActive,
    ContactEmailAddress,
    ClientId,
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
    pub is_active: bool,
    pub contact_email_address: Option<String>,
    pub client_id: Option<String>,
}

impl Shop {
    const SHOP_CUSTOMIZATION_ALIAS: &'static str = "shop_customization";
    const NAME_TS_RANK_ALIAS: &'static str = "name_ts_rank";
    const DESCRIPTION_TS_RANK_ALIAS: &'static str = "description_ts_rank";

    fn get_shop_customization_alias() -> Alias {
        Alias::new(Self::SHOP_CUSTOMIZATION_ALIAS)
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
            Name => {
                let column = (ShopIden::Table, ShopIden::NameTs);
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
                let column = (ShopIden::Table, ShopIden::DescriptionTs);
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
                let name_col = (ShopIden::Table, ShopIden::NameTs);
                let description_col =
                    (ShopIden::Table, ShopIden::DescriptionTs);

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
        }
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
        user_id: Option<&String>,
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
        .cond_where(all![
            Expr::col((ShopIden::Table, ShopIden::ShopId)).eq(*shop_id),
            any![
                Expr::col((ShopIden::Table, ShopIden::IsActive)).eq(true),
                Expr::col((ShopIden::Table, ShopIden::UserId))
                    .eq(user_id.cloned())
            ]
        ])
        .build_postgres(PostgresQueryBuilder);

        Ok(client
            .query_opt(sql.as_str(), &values.as_params())
            .await?
            .map(Self::from))
    }

    pub async fn get_by_slug(
        pool: &Pool,
        slug: &String,
        user_id: Option<&String>,
        extended: bool,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = if extended {
            Self::select_with_relations()
        } else {
            Query::select()
                .column((ShopIden::Table, Asterisk))
                .from(ShopIden::Table)
                .to_owned()
        }
        .cond_where(all![
            Expr::col((ShopIden::Table, ShopIden::Slug)).eq(slug),
            any![
                Expr::col((ShopIden::Table, ShopIden::IsActive)).eq(true),
                Expr::col((ShopIden::Table, ShopIden::UserId))
                    .eq(user_id.cloned())
            ]
        ])
        .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn get_by_domain(
        pool: &Pool,
        domain: &String,
        user_id: Option<&String>,
        extended: bool,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = if extended {
            Self::select_with_relations()
        } else {
            Query::select()
                .column((ShopIden::Table, Asterisk))
                .from(ShopIden::Table)
                .to_owned()
        }
        .cond_where(all![
            Expr::col((ShopIden::Table, ShopIden::Domain)).eq(domain),
            any![
                Expr::col((ShopIden::Table, ShopIden::IsActive)).eq(true),
                Expr::col((ShopIden::Table, ShopIden::UserId))
                    .eq(user_id.cloned())
            ]
        ])
        .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list(
        pool: &Pool,
        user_id: Option<&String>,
        limit: u64,
        offset: u64,
        filter: Option<(ShopsFilterField, String)>,
        order_by: Option<(ShopsOrderByField, Direction)>,
        extended: bool,
        request_user_id: Option<&String>,
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
                query.cond_where(
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

            query.cond_where(any![
                Expr::col((ShopIden::Table, ShopIden::IsActive)).eq(true),
                Expr::col((ShopIden::Table, ShopIden::UserId))
                    .eq(request_user_id.cloned())
            ]);

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
        is_active: Option<bool>,
        contact_email_address: Option<String>,
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

            if let Some(is_active) = is_active {
                query.value(ShopIden::IsActive, is_active);
            }

            if let Some(contact_email_address) = contact_email_address {
                query.value(
                    ShopIden::ContactEmailAddress,
                    contact_email_address,
                );
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
            is_active: row.get(ShopIden::IsActive.to_string().as_str()),
            contact_email_address: row
                .get(ShopIden::ContactEmailAddress.to_string().as_str()),
            client_id: row.get(ShopIden::ClientId.to_string().as_str()),
        }
    }
}

impl From<Row> for Shop {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}
