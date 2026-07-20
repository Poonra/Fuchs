use sqlx::SqlitePool;
use yahoo_finance_api as yahoo;
use time::OffsetDateTime;
use time::Duration;
use crate::db;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration as StdDuration, Instant};

#[derive(Debug, Clone, serde::Serialize)]
pub struct IntradayBar {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

struct CachedIntraday {
    fetched_at: Instant,
    bars: Vec<IntradayBar>,
}

// ponytail: single global mutex-guarded map, no eviction — fine for a
// personal watchlist of a dozen symbols, would need real eviction for
// anything bigger.
static INTRADAY_CACHE: OnceLock<Mutex<HashMap<String, CachedIntraday>>> = OnceLock::new();
const INTRADAY_CACHE_TTL: StdDuration = StdDuration::from_secs(300); // matches the 5m bar interval — no point refetching sooner

// ponytail: live proxy, not persisted to SQLite — intraday bars are only
// useful for the last few sessions, so an in-memory TTL cache is enough;
// no reason for a table alongside the daily `prices` history.
// range=5d (not 1d): Yahoo's "1d" range only covers the current session,
// which is thin-to-empty outside market hours or early in the trading day.
pub async fn fetch_intraday(symbol: &str) -> anyhow::Result<Vec<IntradayBar>> {
    let cache = INTRADAY_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Some(entry) = cache.lock().unwrap().get(symbol) {
        if entry.fetched_at.elapsed() < INTRADAY_CACHE_TTL {
            return Ok(entry.bars.clone());
        }
    }

    let provider = yahoo::YahooConnector::new()?;
    let response = provider.get_quote_range(symbol, "5m", "5d").await?;
    let quotes = response.quotes()?;

    let bars: Vec<IntradayBar> = quotes.into_iter().map(|q| IntradayBar {
        time: q.timestamp,
        open: q.open,
        high: q.high,
        low: q.low,
        close: q.close,
    }).collect();

    cache.lock().unwrap().insert(symbol.to_string(), CachedIntraday {
        fetched_at: Instant::now(),
        bars: bars.clone(),
    });

    Ok(bars)
}


#[cfg(test)]
mod intraday_cache_tests {
    use super::*;

    #[tokio::test]
    async fn cache_hit_skips_network_and_returns_stored_bars() {
        let symbol = "__INTRADAY_CACHE_TEST__";
        let cache = INTRADAY_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        let sentinel = vec![IntradayBar { time: 42, open: 1.0, high: 2.0, low: 0.5, close: 1.5 }];
        cache.lock().unwrap().insert(symbol.to_string(), CachedIntraday {
            fetched_at: Instant::now(),
            bars: sentinel.clone(),
        });

        // A fresh cache entry must short-circuit before any network call —
        // if this reached the network it'd fail resolving the fake symbol.
        let result = fetch_intraday(symbol).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].time, 42);
        assert_eq!(result[0].close, 1.5);
    }
}

pub async fn sync_stock(pool: &SqlitePool, symbol: &str) -> anyhow::Result<()> {
    let stock = match db::get_stock_by_symbol(pool, symbol).await? {
        Some(s) => s,
        None => {
            println!("Stock {} is not in the watchlist. add it first. ", symbol);
            return Ok(())
        }
    };

    let provider = yahoo::YahooConnector::new()?;

    let end = OffsetDateTime::now_utc();
    let start = end - Duration::days(1825);

    let response = provider.get_quote_history(symbol, start, end).await?;
    let quotes = response.quotes()?;

    let search = provider.search_ticker(symbol).await?;
    if let Some(result) = search.quotes.first() {
        db::update_stock_name(pool, symbol, &result.long_name).await?;
    }

    for quote in &quotes {
        let date = OffsetDateTime::from_unix_timestamp(quote.timestamp as i64)?;
        let date_str = format!("{}-{:02}-{:02}", date.year(), date.month() as u8, date.day());

        db::insert_price(
            pool,
            stock.id,
            &date_str,
            quote.open,
            quote.high,
            quote.low,
            quote.close,
            quote.volume as i64,
        ).await?;

    }
    println!("Synced {} prices for {}", quotes.len(), symbol);

    Ok(())
}

