use sqlx::SqlitePool;
use crate::models::{Stock, Price, Fundamentals};

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
        "CREATE TABLE IF NOT EXISTS prices(
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

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS fundamentals (
    id INTEGER PRIMARY KEY,
    stock_id INTEGER NOT NULL,
    as_of_date TEXT NOT NULL,       -- YYYY-MM-DD when we fetched it
    sector TEXT,
    market_cap REAL,
    pe_ratio REAL,
    peg_ratio REAL,
    pb_ratio REAL,
    eps REAL,
    dividend_yield REAL,
    profit_margin REAL,
    roe REAL,                        -- ReturnOnEquityTTM
    debt_to_equity REAL,
    revenue_ttm REAL,
    quarterly_revenue_growth REAL,
    quarterly_earnings_growth REAL,
    FOREIGN KEY(stock_id) REFERENCES stocks(id),
    UNIQUE(stock_id, as_of_date)
);"
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
    sqlx::query("INSERT OR IGNORE INTO prices
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

pub async fn get_prices(pool: &SqlitePool, stock_id:i64, as_of: Option<&str>,) -> Result<Vec<Price>, sqlx::Error> {
    let prices = match as_of {
        Some(date) => {
            sqlx::query_as::<_, Price>(
                "SELECT * FROM prices WHERE stock_id = ? AND date <= ? ORDER BY date ASC"
            )
                .bind(stock_id)
                .bind(date)
                .fetch_all(pool)
                .await?
        }
    None => { sqlx::query_as::<_, Price>(
        "SELECT * FROM prices WHERE stock_id = ? ORDER BY date ASC"
        )
        .bind(stock_id)
        .fetch_all(pool)
        .await?
        }
    };
    Ok(prices)
}

pub async fn insert_fundamentals(
    pool: &SqlitePool,
    stock_id: i64,
    as_of_date: &str,
    sector: Option<&str>,
    market_cap: Option<f64>,
    pe_ratio: Option<f64>,
    peg_ratio: Option<f64>,
    pb_ratio: Option<f64>,
    eps: Option<f64>,
    dividend_yield: Option<f64>,
    profit_margin: Option<f64>,
    roe: Option<f64>,
    debt_to_equity: Option<f64>,
    revenue_ttm: Option<f64>,
    quarterly_revenue_growth: Option<f64>,
    quarterly_earnings_growth: Option<f64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO fundamentals (
            stock_id, as_of_date, sector, market_cap, pe_ratio, peg_ratio,
            pb_ratio, eps, dividend_yield, profit_margin, roe,
            debt_to_equity, revenue_ttm, quarterly_revenue_growth,
            quarterly_earnings_growth
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
        .bind(stock_id)
        .bind(as_of_date)
        .bind(sector)
        .bind(market_cap)
        .bind(pe_ratio)
        .bind(peg_ratio)
        .bind(pb_ratio)
        .bind(eps)
        .bind(dividend_yield)
        .bind(profit_margin)
        .bind(roe)
        .bind(debt_to_equity)
        .bind(revenue_ttm)
        .bind(quarterly_revenue_growth)
        .bind(quarterly_earnings_growth)

        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_latest_fundamentals(
    pool: &SqlitePool,
    stock_id: i64,
) -> Result<Option<Fundamentals>, sqlx::Error> {
    let result = sqlx::query_as::<_, Fundamentals>(
        "SELECT * FROM fundamentals WHERE stock_id = ? ORDER BY as_of_date DESC LIMIT 1"
    )
        .bind(stock_id)
        .fetch_optional(pool)
        .await?;
    Ok(result)
}


pub async fn get_fundamentals_age_days(
    pool: &SqlitePool,
    stock_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT as_of_date FROM fundamentals WHERE stock_id = ? ORDER BY as_of_date DESC LIMIT 1"
    )
        .bind(stock_id)
        .fetch_optional(pool)
        .await?;

    let Some((date_str,)) = row else { return Ok(None); };

    // Parse YYYY-MM-DD and compute days from today
    use time::OffsetDateTime;
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return Ok(None);
    }
    let year: i32 = match parts[0].parse() { Ok(v) => v, Err(_) => return Ok(None) };
    let month: u8 = match parts[1].parse() { Ok(v) => v, Err(_) => return Ok(None) };
    let day: u8 = match parts[2].parse() { Ok(v) => v, Err(_) => return Ok(None) };

    let stored = match time::Date::from_calendar_date(
        year,
        time::Month::try_from(month).map_err(|_| sqlx::Error::Decode("bad month".into()))?,
        day,
    ) {
        Ok(d) => d,
        Err(_) => return Ok(None),
    };

    let today = OffsetDateTime::now_utc().date();
    let diff = today - stored;
    Ok(Some(diff.whole_days()))
}