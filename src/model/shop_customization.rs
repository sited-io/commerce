use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::types::{private, FromSql, Type};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::{Pool, Transaction};
use fallible_iterator::FallibleIterator;
use postgres_protocol::types;
use sea_query::{
    Asterisk, Expr, Func, Iden, OnConflict, PostgresQueryBuilder, Query,
    SimpleExpr,
};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::db::{get_type_from_oid, ArrayAgg, DbError};

#[derive(Debug, Clone, Copy, Iden)]
#[iden(rename = "shop_customizations")]
pub enum ShopCustomizationIden {
    Table,
    ShopId,
    UserId,
    CreatedAt,
    UpdatedAt,
    LogoImageLightUrlPath,
    LogoImageDarkUrlPath,
    BannerImageLightUrlPath,
    BannerImageDarkUrlPath,
    PrimaryColor,
    LayoutType,
}

#[derive(Debug, Clone)]
pub struct ShopCustomization {
    pub shop_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub logo_image_light_url_path: Option<String>,
    pub logo_image_dark_url_path: Option<String>,
    pub banner_image_light_url_path: Option<String>,
    pub banner_image_dark_url_path: Option<String>,
    pub primary_color: Option<String>,
    pub layout_type: String,
}

impl ShopCustomization {
    const PUT_COLUMNS: [ShopCustomizationIden; 4] = [
        ShopCustomizationIden::ShopId,
        ShopCustomizationIden::UserId,
        ShopCustomizationIden::PrimaryColor,
        ShopCustomizationIden::LayoutType,
    ];

    pub async fn create(
        pool: &Pool,
        shop_id: &Uuid,
        user_id: &String,
    ) -> Result<(), DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(ShopCustomizationIden::Table)
            .columns([
                ShopCustomizationIden::ShopId,
                ShopCustomizationIden::UserId,
            ])
            .values([(*shop_id).into(), user_id.into()])?
            .on_conflict(
                OnConflict::column(ShopCustomizationIden::ShopId)
                    .do_nothing()
                    .to_owned(),
            )
            .build_postgres(PostgresQueryBuilder);

        conn.execute(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn put(
        pool: &Pool,
        shop_id: &Uuid,
        user_id: &String,
        primary_color: Option<String>,
        layout_type: String,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(ShopCustomizationIden::Table)
            .columns(Self::PUT_COLUMNS)
            .values([
                (*shop_id).into(),
                user_id.into(),
                primary_color.into(),
                layout_type.into(),
            ])?
            .on_conflict(
                OnConflict::column(ShopCustomizationIden::ShopId)
                    .update_columns(Self::PUT_COLUMNS)
                    .to_owned(),
            )
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn get(
        pool: &Pool,
        shop_id: &Uuid,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(ShopCustomizationIden::Table)
            .and_where(Expr::col(ShopCustomizationIden::ShopId).eq(*shop_id))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn update_logo_image_url_paths<'a>(
        transaction: &Transaction<'a>,
        shop_id: &Uuid,
        user_id: &String,
        logo_image_light_url_path: Option<Option<String>>,
        logo_image_dark_url_path: Option<Option<String>>,
    ) -> Result<Self, DbError> {
        let (sql, values) = {
            let mut query = Query::update();

            query.table(ShopCustomizationIden::Table);

            if let Some(logo_image_light_url_path) = logo_image_light_url_path {
                query.value(
                    ShopCustomizationIden::LogoImageLightUrlPath,
                    logo_image_light_url_path,
                );
            }

            if let Some(logo_image_dark_url_path) = logo_image_dark_url_path {
                query.value(
                    ShopCustomizationIden::LogoImageDarkUrlPath,
                    logo_image_dark_url_path,
                );
            }

            query
                .and_where(Expr::col(ShopCustomizationIden::UserId).eq(user_id))
                .and_where(
                    Expr::col(ShopCustomizationIden::ShopId).eq(*shop_id),
                )
                .returning_all()
                .build_postgres(PostgresQueryBuilder)
        };

        let row = transaction
            .query_one(sql.as_str(), &values.as_params())
            .await?;

        Ok(Self::from(row))
    }

    pub async fn update_banner_image_url_paths<'a>(
        transaction: &Transaction<'a>,
        shop_id: &Uuid,
        user_id: &String,
        banner_image_light_url_path: Option<Option<String>>,
        banner_image_dark_url_path: Option<Option<String>>,
    ) -> Result<Self, DbError> {
        let (sql, values) = {
            let mut query = Query::update();

            query.table(ShopCustomizationIden::Table);

            if let Some(banner_image_light_url_path) =
                banner_image_light_url_path
            {
                query.value(
                    ShopCustomizationIden::BannerImageLightUrlPath,
                    banner_image_light_url_path,
                );
            }

            if let Some(banner_image_dark_url_path) = banner_image_dark_url_path
            {
                query.value(
                    ShopCustomizationIden::BannerImageDarkUrlPath,
                    banner_image_dark_url_path,
                );
            }

            query
                .and_where(Expr::col(ShopCustomizationIden::UserId).eq(user_id))
                .and_where(
                    Expr::col(ShopCustomizationIden::ShopId).eq(*shop_id),
                )
                .returning_all()
                .build_postgres(PostgresQueryBuilder)
        };

        let row = transaction
            .query_one(sql.as_str(), &values.as_params())
            .await?;

        Ok(Self::from(row))
    }

    pub async fn delete<'a>(
        transaction: &Transaction<'a>,
        shop_id: &Uuid,
        user_id: &String,
    ) -> Result<(), DbError> {
        let (sql, values) = Query::delete()
            .from_table(ShopCustomizationIden::Table)
            .and_where(Expr::col(ShopCustomizationIden::UserId).eq(user_id))
            .and_where(Expr::col(ShopCustomizationIden::ShopId).eq(*shop_id))
            .build_postgres(PostgresQueryBuilder);

        transaction.query(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }
}

impl From<&Row> for ShopCustomization {
    fn from(row: &Row) -> Self {
        Self {
            shop_id: row
                .get(ShopCustomizationIden::ShopId.to_string().as_str()),
            user_id: row
                .get(ShopCustomizationIden::UserId.to_string().as_str()),
            created_at: row
                .get(ShopCustomizationIden::CreatedAt.to_string().as_str()),
            updated_at: row
                .get(ShopCustomizationIden::UpdatedAt.to_string().as_str()),
            logo_image_light_url_path: row.get(
                ShopCustomizationIden::LogoImageLightUrlPath
                    .to_string()
                    .as_str(),
            ),
            logo_image_dark_url_path: row.get(
                ShopCustomizationIden::LogoImageDarkUrlPath
                    .to_string()
                    .as_str(),
            ),
            banner_image_light_url_path: row.get(
                ShopCustomizationIden::BannerImageLightUrlPath
                    .to_string()
                    .as_str(),
            ),
            banner_image_dark_url_path: row.get(
                ShopCustomizationIden::BannerImageDarkUrlPath
                    .to_string()
                    .as_str(),
            ),
            primary_color: row
                .get(ShopCustomizationIden::PrimaryColor.to_string().as_str()),
            layout_type: row
                .get(ShopCustomizationIden::LayoutType.to_string().as_str()),
        }
    }
}

impl From<Row> for ShopCustomization {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}

#[derive(Debug, Clone)]
pub struct ShopCustomizationAsRel {
    pub logo_image_light_url_path: Option<String>,
    pub logo_image_dark_url_path: Option<String>,
    pub banner_image_light_url_path: Option<String>,
    pub banner_image_dark_url_path: Option<String>,
    pub primary_color: Option<String>,
    pub layout_type: String,
}

impl ShopCustomizationAsRel {
    pub fn get_agg() -> SimpleExpr {
        Func::cust(ArrayAgg)
            .args([Expr::tuple([
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::LogoImageLightUrlPath,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::LogoImageDarkUrlPath,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::BannerImageLightUrlPath,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::BannerImageDarkUrlPath,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::PrimaryColor,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::LayoutType,
                ))
                .into(),
            ])
            .into()])
            .into()
    }
}

impl<'a> FromSql<'a> for ShopCustomizationAsRel {
    fn accepts(ty: &deadpool_postgres::tokio_postgres::types::Type) -> bool {
        matches!(*ty, Type::RECORD)
    }

    fn from_sql(
        _: &Type,
        mut raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        private::read_be_i32(&mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let logo_image_light_url_path: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let logo_image_dark_url_path: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let banner_image_light_url_path: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let banner_image_dark_url_path: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let primary_color: Option<String> = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<String>(oid)?;
        let layout_type: String = private::read_value(&ty, &mut raw)?;

        Ok(Self {
            logo_image_light_url_path,
            logo_image_dark_url_path,
            banner_image_light_url_path,
            banner_image_dark_url_path,
            primary_color,
            layout_type,
        })
    }
}

#[derive(Debug)]
pub struct ShopCustomizationAsRelVec(pub Vec<ShopCustomizationAsRel>);

impl<'a> FromSql<'a> for ShopCustomizationAsRelVec {
    fn accepts(ty: &Type) -> bool {
        match *ty {
            Type::RECORD_ARRAY => true,
            _ => {
                tracing::log::error!("[ShopCustomizationAsRelVec::<FromSql>::accepts]: postgres type {:?} not implemented", ty);
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
                    Ok(ShopCustomizationAsRel::from_sql_nullable(
                        &Type::RECORD,
                        v,
                    )
                    .ok())
                })
                .collect()?,
        ))
    }
}
