use chrono::{DateTime, Utc};
use deadpool_postgres::{tokio_postgres::Row, Pool};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::{
    api::peoplesmarkets::commerce::v1::ShopResponse,
    db::{DbError, Pagination},
};

#[derive(Iden)]
#[iden = "shop"]
pub enum ShopIden {
    Table,
    ShopId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Name,
    Description,
}

const SHOP_COLUMNS: [ShopIden; 6] = [
    ShopIden::ShopId,
    ShopIden::UserId,
    ShopIden::CreatedAt,
    ShopIden::UpdatedAt,
    ShopIden::Name,
    ShopIden::Description,
];

#[derive(Debug, Clone)]
pub struct Shop {
    pub shop_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
}

impl Into<ShopResponse> for Shop {
    fn into(self) -> ShopResponse {
        ShopResponse {
            shop_id: self.shop_id.to_string(),
            user_id: self.user_id,
            created_at: self.created_at.timestamp(),
            updated_at: self.updated_at.timestamp(),
            name: self.name,
            description: self.description,
        }
    }
}

impl From<&Row> for Shop {
    fn from(row: &Row) -> Self {
        Self {
            shop_id: row.get("shop_id"),
            user_id: row.get("user_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            name: row.get("name"),
            description: row.get("description"),
        }
    }
}

impl From<Row> for Shop {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}

pub async fn create_shop(
    pool: &Pool,
    user_id: &String,
    name: &String,
    description: &String,
) -> Result<Shop, DbError> {
    let client = pool.get().await?;

    let shop_id = Uuid::new_v4();
    let now = Utc::now();

    let (sql, values) = Query::insert()
        .into_table(ShopIden::Table)
        .columns(SHOP_COLUMNS)
        .values([
            shop_id.into(),
            user_id.into(),
            now.into(),
            now.into(),
            name.into(),
            description.into(),
        ])?
        .returning_all()
        .build_postgres(PostgresQueryBuilder);

    let row = client.query_one(sql.as_str(), &values.as_params()).await?;

    Ok(Shop::from(&row))
}

pub async fn get_shop(
    pool: &Pool,
    shop_id: &Uuid,
) -> Result<Option<Shop>, DbError> {
    let client = pool.get().await?;

    let (sql, values) = Query::select()
        .columns(SHOP_COLUMNS)
        .from(ShopIden::Table)
        .and_where(Expr::col(ShopIden::ShopId).eq(*shop_id))
        .build_postgres(PostgresQueryBuilder);

    let row = client.query_opt(sql.as_str(), &values.as_params()).await?;

    Ok(row.map(Shop::from))
}

pub async fn list_shops(
    pool: &Pool,
    user_id: Option<&String>,
) -> Result<(Vec<Shop>, Pagination), DbError> {
    let client = pool.get().await?;

    let mut query = Query::select();

    query.columns(SHOP_COLUMNS).from(ShopIden::Table);

    if let Some(user_id) = user_id {
        query.and_where(Expr::col(ShopIden::UserId).eq(user_id));
    }

    let (sql, values) = query.build_postgres(PostgresQueryBuilder);

    let rows = client.query(sql.as_str(), &values.as_params()).await?;

    Ok((
        rows.iter().map(Shop::from).collect(),
        Pagination { page: 1, size: 1 },
    ))
}

pub async fn update_shop(
    pool: &Pool,
    shop_id: &Uuid,
    name: Option<&String>,
    description: Option<&String>,
) -> Result<Shop, DbError> {
    let client = pool.get().await?;

    let mut query = Query::update();
    query.table(ShopIden::Table);

    if let Some(name) = name {
        query.value(ShopIden::Name, name);
    }

    if let Some(description) = description {
        query.value(ShopIden::Description, description);
    }

    query
        .and_where(Expr::col(ShopIden::ShopId).eq(*shop_id))
        .returning_all();

    let (sql, values) = query.build_postgres(PostgresQueryBuilder);

    let row = client.query_one(sql.as_str(), &values.as_params()).await?;

    Ok(Shop::from(row))
}

pub async fn delete_shop(pool: &Pool, shop_id: &Uuid) -> Result<(), DbError> {
    let client = pool.get().await?;

    let (sql, values) = Query::delete()
        .from_table(ShopIden::Table)
        .and_where(Expr::col(ShopIden::ShopId).eq(*shop_id))
        .build_postgres(PostgresQueryBuilder);

    client.execute(sql.as_str(), &values.as_params()).await?;

    Ok(())
}
