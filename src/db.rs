use sqlx::SqlitePool;

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
