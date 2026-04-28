use sqlx::SqlitePool;
use yahoo_finance_api as yahoo;
use time::OffsetDateTime;
use time::Duration;
use crate::db;


pub async fn sync_stock(pool: &SqlitePool, symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
    let stock = match db::get_stock_by_symbol(pool, symbol).await? {
        Some(s) => s,
        None => {
            println!("Stock {} is not in the watchlist. add it first. ", symbol);
            return Ok(())
        }
    };

    let provider = yahoo::YahooConnector::new()?;

    let end = OffsetDateTime::now_utc();
    let start = end - Duration::days(730);

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

