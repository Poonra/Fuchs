use sqlx::SqlitePool;
use crate::models::Stock;
pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {

    let pool = SqlitePool::connect("sqlite:fuchs.db?mode=rwc").await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS stocks(
        id INTEGER PRIMARY KEY,
        symbol TEXT NOT NULL UNIQUE,
        name TEXT
           )"
)
.execute(&pool)
.await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS price(
        id INTEGER PRIMARY KEY,
        stock_id INTEGER NOT NULL,
        date TEXT NOT NULL,
        open REAL,
        high REAL,
        low REAL,
        close REAL,
        volume INTEGER,

        FOREIGN KEY(stock_id) REFERENCES stocks(id),
        UNIQUE(stock_id, date)
           )"
)
.execute(&pool)
.await?;

Ok(pool)

}

pub async fn  add_stock(pool: &SqlitePool, symbol: &str) -> Result<(), sqlx::Error>{

    let upper = symbol.to_uppercase();
    sqlx::query("INSERT INTO stocks (symbol) VALUES (?)")
        .bind(&upper)
        .execute(pool)
        .await?;
    Ok(())
}
pub async fn  list_stocks(pool: &SqlitePool) -> Result<Vec<Stock>, sqlx::Error>{

    let lists = sqlx::query_as::<_, Stock>("SELECT id, symbol, name FROM stocks")
        .fetch_all(pool)
        .await?;
    Ok(lists)
}

pub async fn get_stock_by_symbol(pool: &SqlitePool, symbol: &str) -> Result<Option<Stock>, sqlx::Error> {
    let stock_name = sqlx::query_as::<_, Stock>("SELECT id, symbol, name FROM stocks where symbol = ?")
        .bind(&symbol)
        .fetch_optional(pool)
        .await?;
    Ok(stock_name)
}

pub async fn insert_price(pool: &SqlitePool, stock_id: i64,
                          date: &str, open: f64,
                          high: f64, low: f64,
                          close: f64, volume: i64) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO price
                    (stock_id, date, open, high, low, close, volume) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(stock_id)
        .bind(date)
        .bind(open)
        .bind(high)
        .bind(low)
        .bind(close)
        .bind(volume)
        .execute(pool).await?;

    Ok(())

}

pub async fn update_stock_name(pool: &SqlitePool, symbol: &str, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE stocks SET name = ? WHERE symbol = ?")
        .bind(name)
        .bind(symbol)
        .execute(pool)
        .await?;
    Ok(())
}

