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

