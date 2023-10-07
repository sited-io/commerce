use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::Pool;
use sea_query::{
    all, Asterisk, Expr, Func, Iden, Order, PostgresQueryBuilder, Query,
    SelectStatement,
};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::api::peoplesmarkets::commerce::v1::ShippingRatesOrderByField;
use crate::api::peoplesmarkets::ordering::v1::Direction;
use crate::db::DbError;

#[derive(Iden)]
#[iden(rename = "shipping_rates")]
pub enum ShippingRateIden {
    Table,
    ShippingRateId,
    OfferId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Country,
    Amount,
    Currency,
}

#[derive(Debug, Clone)]
pub struct ShippingRate {
    pub shipping_rate_id: Uuid,
    pub offer_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub country: String,
    pub amount: u32,
    pub currency: String,
}

impl ShippingRate {
    fn add_order_by(
        query: &mut SelectStatement,
        order_by_field: ShippingRatesOrderByField,
        order_by_direction: Direction,
    ) {
        use ShippingRatesOrderByField::*;

        let order = match order_by_direction {
            Direction::Unspecified | Direction::Asc => Order::Asc,
            Direction::Desc => Order::Desc,
        };

        match order_by_field {
            Unspecified | Country => {
                query.order_by(ShippingRateIden::Country, order);
            }
        }
    }

    pub async fn create(
        pool: &Pool,
        offer_id: &Uuid,
        user_id: &String,
        country: &str,
        amount: u32,
        currency: &str,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(ShippingRateIden::Table)
            .columns([
                ShippingRateIden::OfferId,
                ShippingRateIden::UserId,
                ShippingRateIden::Country,
                ShippingRateIden::Amount,
                ShippingRateIden::Currency,
            ])
            .values([
                (*offer_id).into(),
                user_id.into(),
                country.into(),
                i64::from(amount).into(),
                currency.into(),
            ])?
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn list(
        pool: &Pool,
        offer_id: &Uuid,
        limit: u32,
        offset: u32,
        order_by: Option<(ShippingRatesOrderByField, Direction)>,
    ) -> Result<(Vec<Self>, u32), DbError> {
        let conn = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::select();
            query
                .column(Asterisk)
                .from(ShippingRateIden::Table)
                .cond_where(Expr::col(ShippingRateIden::OfferId).eq(*offer_id));

            if let Some((order_by_field, order_by_direction)) = order_by {
                Self::add_order_by(
                    &mut query,
                    order_by_field,
                    order_by_direction,
                );
            }

            query
                .limit(limit.into())
                .offset(offset.into())
                .build_postgres(PostgresQueryBuilder)
        };

        let rows = conn.query(sql.as_str(), &values.as_params()).await?;
        let res = rows.iter().map(Self::from).collect();

        let (sql, values) = Query::select()
            .expr(Func::count(Expr::col(Asterisk)))
            .from(ShippingRateIden::Table)
            .cond_where(Expr::col(ShippingRateIden::OfferId).eq(*offer_id))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;
        let count = u32::try_from(row.try_get::<usize, i64>(0)?).unwrap();

        Ok((res, count))
    }

    pub async fn delete(
        pool: &Pool,
        shipping_rate_id: &Uuid,
        offer_id: &Uuid,
        user_id: &String,
    ) -> Result<(), DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::delete()
            .from_table(ShippingRateIden::Table)
            .cond_where(all![
                Expr::col(ShippingRateIden::ShippingRateId)
                    .eq(*shipping_rate_id),
                Expr::col(ShippingRateIden::OfferId).eq(*offer_id),
                Expr::col(ShippingRateIden::UserId).eq(user_id)
            ])
            .build_postgres(PostgresQueryBuilder);

        conn.execute(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }
}

impl From<&Row> for ShippingRate {
    fn from(row: &Row) -> Self {
        Self {
            shipping_rate_id: row
                .get(ShippingRateIden::ShippingRateId.to_string().as_str()),
            offer_id: row.get(ShippingRateIden::OfferId.to_string().as_str()),
            user_id: row.get(ShippingRateIden::UserId.to_string().as_str()),
            created_at: row
                .get(ShippingRateIden::CreatedAt.to_string().as_str()),
            updated_at: row
                .get(ShippingRateIden::UpdatedAt.to_string().as_str()),
            country: row.get(ShippingRateIden::Country.to_string().as_str()),
            amount: u32::try_from(row.get::<&str, i64>(
                ShippingRateIden::Amount.to_string().as_str(),
            ))
            .unwrap(),
            currency: row.get(ShippingRateIden::Currency.to_string().as_str()),
        }
    }
}

impl From<Row> for ShippingRate {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}
