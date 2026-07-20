use sqlx::SqlitePool;
use crate::models::{Stock, Price, Fundamentals, Earnings};

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:C:/Users/poonr/Desktop/Project/Fuchs/fuchs.db?mode=rwc".to_string());
    let pool = SqlitePool::connect(&db_url).await?;

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

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS score_history (
    id INTEGER PRIMARY KEY,
    stock_id INTEGER NOT NULL,
    date TEXT NOT NULL,       -- YYYY-MM-DD when we fetched it
    technical INTEGER NOT NULL,
    fundamental INTEGER,
    combined INTEGER NOT NULL,
    FOREIGN KEY(stock_id) REFERENCES stocks(id),
    UNIQUE(stock_id, date)
);"
    )
        .execute(&pool)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS earnings (
    id INTEGER PRIMARY KEY,
    stock_id INTEGER NOT NULL,
    fiscal_date TEXT NOT NULL,      -- quarter end, YYYY-MM-DD
    reported_eps REAL,
    estimated_eps REAL,
    surprise_pct REAL,              -- AV surprisePercentage, e.g. 4.53 = beat by 4.53%
    FOREIGN KEY(stock_id) REFERENCES stocks(id),
    UNIQUE(stock_id, fiscal_date)
);"
    )
        .execute(&pool)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS news_sources (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,      -- 'SET disclosures', 'Kaohoon', ...
    kind TEXT NOT NULL,             -- 'rss' | 'scrape' | 'line'
    url TEXT,                       -- feed/endpoint URL, NULL for 'line'
    enabled INTEGER NOT NULL DEFAULT 1
);"
    )
        .execute(&pool)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS articles (
    id INTEGER PRIMARY KEY,
    source_id INTEGER NOT NULL,
    url TEXT NOT NULL,
    url_hash TEXT NOT NULL UNIQUE,  -- sha256(url) = dedup key
    title TEXT NOT NULL,
    body TEXT,                      -- FULL text, kept for later ML
    published_at TEXT,              -- ISO timestamp from the source
    fetched_at TEXT NOT NULL,       -- when WE stored it
    FOREIGN KEY(source_id) REFERENCES news_sources(id)
);"
    )
        .execute(&pool)
        .await?;

sqlx::query(
    "CREATE TABLE IF NOT EXISTS article_tickers (
    id INTEGER PRIMARY KEY,
    article_id INTEGER NOT NULL,
    symbol TEXT NOT NULL,
    sentiment INTEGER,              -- -1/0/+1, NULL until LLM-tagged
    relevance REAL,                 -- NULL until tagged
    summary TEXT,                   -- NULL until tagged
    FOREIGN KEY(article_id) REFERENCES articles(id),
    UNIQUE(article_id, symbol)
);"
)
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ticker_aliases (
    id INTEGER PRIMARY KEY,
    symbol TEXT NOT NULL,
    alias TEXT NOT NULL,
    UNIQUE(symbol, alias)
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
pub async fn delete_stock(pool: &SqlitePool, symbol: &str) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let stock_id: Option<i64> = sqlx::query_scalar("SELECT id FROM stocks WHERE symbol = ?")
        .bind(symbol)
        .fetch_optional(&mut *tx)
        .await?;

    let Some(stock_id) = stock_id else {
        return Ok(false);
    };

    sqlx::query("DELETE FROM prices WHERE stock_id = ?").bind(stock_id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM fundamentals WHERE stock_id = ?").bind(stock_id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM earnings WHERE stock_id = ?").bind(stock_id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM score_history WHERE stock_id = ?").bind(stock_id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM stocks WHERE id = ?").bind(stock_id).execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(true)
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

pub async fn insert_earnings(
    pool: &SqlitePool,
    stock_id: i64,
    fiscal_date: &str,
    reported_eps: Option<f64>,
    estimated_eps: Option<f64>,
    surprise_pct: Option<f64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO earnings
            (stock_id, fiscal_date, reported_eps, estimated_eps, surprise_pct)
         VALUES (?, ?, ?, ?, ?)"
    )
        .bind(stock_id)
        .bind(fiscal_date)
        .bind(reported_eps)
        .bind(estimated_eps)
        .bind(surprise_pct)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_recent_earnings(
    pool: &SqlitePool,
    stock_id: i64,
    limit: i64,
) -> Result<Vec<Earnings>, sqlx::Error> {
    sqlx::query_as::<_, Earnings>(
        "SELECT * FROM earnings WHERE stock_id = ? ORDER BY fiscal_date DESC LIMIT ?"
    )
        .bind(stock_id)
        .bind(limit)
        .fetch_all(pool)
        .await
}

pub async fn insert_score_history(pool: &SqlitePool, stock_id: i64,
                          date: &str, technical: i32,
                          fundamental: Option<i32>, combined: i32,
                      ) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO score_history
                    (stock_id, date, technical, fundamental, combined) VALUES (?, ?, ?, ?, ?)")
        .bind(stock_id)
        .bind(date)
        .bind(technical)
        .bind(fundamental)
        .bind(combined)
        .execute(pool).await?;

    Ok(())

}

pub async fn ensure_source(pool: &SqlitePool, name: &str, kind: &str, url: &str) -> Result<i64, sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO news_sources (name, kind, url) VALUES (?, ?, ?)")
        .bind(name).bind(kind).bind(url)
        .execute(pool)
        .await?;

    sqlx::query_scalar("SELECT id FROM news_sources WHERE name = ?")
        .bind(name)
        .fetch_one(pool)
        .await
}
pub async fn insert_article(
pool: &SqlitePool, source_id: i64, url: &str, url_hash: &str,
title: &str, body: Option<&str>, published_at: Option<&str>, fetched_at: &str,
) -> Result<bool, sqlx::Error> {
let res = sqlx::query(
"INSERT OR IGNORE INTO articles (source_id, url, url_hash, title, body, published_at, fetched_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)")
.bind(source_id).bind(url).bind(url_hash)
.bind(title).bind(body).bind(published_at).bind(fetched_at)
.execute(pool)
.await?;
Ok(res.rows_affected() > 0)
}

pub async fn list_articles(pool: &SqlitePool, limit: i64) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT substr(COALESCE(published_at, fetched_at), 1, 10), title
         FROM articles
         ORDER BY COALESCE(published_at, fetched_at) DESC
         LIMIT ?")
        .bind(limit)
        .fetch_all(pool)
        .await
}