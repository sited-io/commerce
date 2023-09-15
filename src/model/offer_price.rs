use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::types::{private, FromSql, Type};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::Pool;
use fallible_iterator::FallibleIterator;
use postgres_protocol::types;
use sea_query::{
    Asterisk, Expr, Func, Iden, PostgresQueryBuilder, Query, SimpleExpr,
};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::db::{get_type_from_oid, ArrayAgg, DbError};

#[derive(Iden)]
#[iden(rename = "offer_prices")]
pub enum OfferPriceIden {
    Table,
    OfferPriceId,
    OfferId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Currency,
    PriceType,
    BillingScheme,
    UnitAmount,
    RecurringInterval,
    RecurringIntervalCount,
}

#[derive(Debug, Clone)]
pub struct OfferPrice {
    pub offer_price_id: Uuid,
    pub offer_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub currency: String,
    pub price_type: String,
    pub billing_scheme: String,
    pub unit_amount: u32,
    pub recurring_interval: Option<String>,
    pub recurring_interval_count: Option<u32>,
}

impl OfferPrice {
    pub async fn create(
        pool: &Pool,
        offer_id: &Uuid,
        user_id: &String,
        currency: &str,
        price_type: &str,
        billing_scheme: &str,
        unit_amount: u32,
        recurring_interval: Option<&str>,
        recurring_interval_count: Option<u32>,
    ) -> Result<Self, DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(OfferPriceIden::Table)
            .columns([
                OfferPriceIden::OfferId,
                OfferPriceIden::UserId,
                OfferPriceIden::Currency,
                OfferPriceIden::PriceType,
                OfferPriceIden::BillingScheme,
                OfferPriceIden::UnitAmount,
                OfferPriceIden::RecurringInterval,
                OfferPriceIden::RecurringIntervalCount,
            ])
            .values([
                (*offer_id).into(),
                user_id.into(),
                currency.into(),
                price_type.into(),
                billing_scheme.into(),
                i64::from(unit_amount).into(),
                recurring_interval.into(),
                recurring_interval_count.map(i64::from).into(),
            ])?
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = client.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn get_by_offer_id(
        pool: &Pool,
        offer_id: &Uuid,
    ) -> Result<Option<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(OfferPriceIden::Table)
            .and_where(Expr::col(OfferPriceIden::OfferId).eq(*offer_id))
            .build_postgres(PostgresQueryBuilder);

        Ok(client
            .query_opt(sql.as_str(), &values.as_params())
            .await?
            .map(Self::from))
    }

    pub async fn put(
        pool: &Pool,
        user_id: &String,
        offer_id: &Uuid,
        currency: &str,
        price_type: &str,
        billing_scheme: &str,
        unit_amount: u32,
        recurring_interval: Option<&str>,
        recurring_interval_count: Option<u32>,
    ) -> Result<Self, DbError> {
        let client = pool.get().await?;

        let (sql, values) = Query::update()
            .table(OfferPriceIden::Table)
            .value(OfferPriceIden::Currency, currency)
            .value(OfferPriceIden::PriceType, price_type)
            .value(OfferPriceIden::BillingScheme, billing_scheme)
            .value(OfferPriceIden::UnitAmount, i64::from(unit_amount))
            .value(OfferPriceIden::RecurringInterval, recurring_interval)
            .value(
                OfferPriceIden::RecurringIntervalCount,
                recurring_interval_count.map(i64::from),
            )
            .and_where(Expr::col(OfferPriceIden::UserId).eq(user_id))
            .and_where(Expr::col(OfferPriceIden::OfferId).eq(*offer_id))
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

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
            .from_table(OfferPriceIden::Table)
            .and_where(Expr::col(OfferPriceIden::UserId).eq(user_id))
            .and_where(Expr::col(OfferPriceIden::OfferId).eq(*offer_id))
            .build_postgres(PostgresQueryBuilder);

        client.execute(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }
}

impl From<&Row> for OfferPrice {
    fn from(row: &Row) -> Self {
        Self {
            offer_price_id: row
                .get(OfferPriceIden::OfferPriceId.to_string().as_str()),
            offer_id: row.get(OfferPriceIden::OfferId.to_string().as_str()),
            user_id: row.get(OfferPriceIden::UserId.to_string().as_str()),
            created_at: row.get(OfferPriceIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(OfferPriceIden::UpdatedAt.to_string().as_str()),
            currency: row.get(OfferPriceIden::Currency.to_string().as_str()),
            price_type: row.get(OfferPriceIden::PriceType.to_string().as_str()),
            billing_scheme: row
                .get(OfferPriceIden::BillingScheme.to_string().as_str()),
            unit_amount: u32::try_from(row.get::<&str, i64>(
                OfferPriceIden::UnitAmount.to_string().as_str(),
            ))
            .expect("Should not be greater than 4294967295"),
            recurring_interval: row
                .get(OfferPriceIden::RecurringInterval.to_string().as_str()),
            recurring_interval_count: row
                .get::<&str, Option<i64>>(
                    OfferPriceIden::RecurringIntervalCount.to_string().as_str(),
                )
                .map(|c| {
                    u32::try_from(c)
                        .expect("Should not be greater than 4294967295")
                }),
        }
    }
}

impl From<Row> for OfferPrice {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}

#[derive(Debug, Clone)]
pub struct OfferPriceAsRel {
    pub offer_price_id: Uuid,
    pub currency: String,
    pub price_type: String,
    pub billing_scheme: String,
    pub unit_amount: u32,
    pub recurring_interval: Option<String>,
    pub recurring_interval_count: Option<u32>,
}

impl OfferPriceAsRel {
    pub fn get_agg() -> SimpleExpr {
        Func::cust(ArrayAgg)
            .args([Expr::tuple([
                Expr::col((
                    OfferPriceIden::Table,
                    OfferPriceIden::OfferPriceId,
                ))
                .into(),
                Expr::col((OfferPriceIden::Table, OfferPriceIden::Currency))
                    .into(),
                Expr::col((OfferPriceIden::Table, OfferPriceIden::PriceType))
                    .into(),
                Expr::col((
                    OfferPriceIden::Table,
                    OfferPriceIden::BillingScheme,
                ))
                .into(),
                Expr::col((OfferPriceIden::Table, OfferPriceIden::UnitAmount))
                    .into(),
                Expr::col((
                    OfferPriceIden::Table,
                    OfferPriceIden::RecurringInterval,
                ))
                .into(),
                Expr::col((
                    OfferPriceIden::Table,
                    OfferPriceIden::RecurringIntervalCount,
                ))
                .into(),
            ])
            .into()])
            .into()
    }
}

impl<'a> FromSql<'a> for OfferPriceAsRel {
    fn accepts(ty: &deadpool_postgres::tokio_postgres::types::Type) -> bool {
        match *ty {
            Type::RECORD => true,
            _ => {
                tracing::log::error!(
                    "[OfferPriceAsRel.FromSql.accepts]: postgres type {:?} not implemented", 
                    ty
                );
                false
            }
        }
    }

    fn from_sql(
        _: &Type,
        mut raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        private::read_be_i32(&mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Uuid>(oid)?;
        let offer_price_id: Uuid = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<String>(oid)?;
        let currency: String = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<String>(oid)?;
        let price_type: String = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<String>(oid)?;
        let billing_scheme: String = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<i64>(oid)?;
        let unit_amount: i64 = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let recurring_interval: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<i64>>(oid)?;
        let recurring_interval_count: Option<i64> =
            private::read_value(&ty, &mut raw)?;

        let recurring_interval_count = match recurring_interval_count {
            Some(c) => Some(u32::try_from(c)?),
            None => None,
        };

        Ok(Self {
            offer_price_id,
            currency,
            price_type,
            billing_scheme,
            unit_amount: u32::try_from(unit_amount)?,
            recurring_interval,
            recurring_interval_count,
        })
    }
}

#[derive(Debug)]
pub struct OfferPriceAsRelVec(pub Vec<OfferPriceAsRel>);

impl<'a> FromSql<'a> for OfferPriceAsRelVec {
    fn accepts(ty: &Type) -> bool {
        match *ty {
            Type::RECORD_ARRAY => true,
            _ => {
                tracing::log::error!("[OfferPriceAsRelVec::<FromSql>::accepts]: postgres type {:?} not implemented", ty);
                false
            }
        }
    }

    fn from_sql(
        _: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let array = types::array_from_sql(raw)?;

        if array.dimensions().count()? > 1 {
            return Err("array contains too many dimensions".into());
        }

        Ok(Self(
            array
                .values()
                .filter_map(|v| {
                    Ok(OfferPriceAsRel::from_sql_nullable(&Type::RECORD, v)
                        .ok())
                })
                .collect()?,
        ))
    }
}
