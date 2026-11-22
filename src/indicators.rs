use sqlx::SqlitePool;
use crate::db::{get_prices, get_stock_by_symbol};
use anyhow::bail;
use ta::Next;
use ta::indicators::{BollingerBands, BollingerBandsOutput, SimpleMovingAverage};
use fin_primitives::signals::batch_f64;

#[derive(Debug, serde::Serialize)]
pub struct Indicators {
    pub rsi: f64,
    pub sma_50: f64,
    pub sma_200: f64,
    pub macd: f64,
    pub macd_signal: f64,
    pub macd_histogram: f64,
    pub bb_lower: f64,
    pub bb_middle: f64,
    pub bb_upper: f64,
    pub last_close: f64,
}

pub async fn compute(pool: &SqlitePool,symbol: &str,as_of: Option<&str>) -> anyhow::Result<Indicators>{
    let stock = get_stock_by_symbol(pool, symbol).await?
        .ok_or_else(|| anyhow::anyhow!("Stock {} not found", symbol))?;
    let prices = get_prices(pool, stock.id, as_of).await?;

    if prices.is_empty() {
            bail!("No prices found for {}. Run `fuchs sync` first.", symbol);
    }
    let mut sma_50 = SimpleMovingAverage::new(50).unwrap();
    let mut latest_sma_50 = 0.0;
    let mut sma_200 = SimpleMovingAverage::new(200).unwrap();
    let mut latest_sma_200 = 0.0;
    let mut bb = BollingerBands::new(20,2.0).unwrap();
    let mut latest_bb = BollingerBandsOutput{
            lower:0.0,
            average:0.0,
            upper:0.0
    };
    let mut last_close = 0.0;

    for price in &prices {
        latest_sma_50 = sma_50.next(price.close);
        latest_sma_200 = sma_200.next(price.close);
        latest_bb = bb.next(price.close);
        last_close = price.close;

    }
    let closes: Vec<f64> = prices.iter().map(|p| p.close).collect();
    let latest_rsi = *batch_f64::rsi(&closes, 14).last().unwrap();
    let (macd_line, macd_signal, macd_histogram) = batch_f64::macd(&closes, 12, 26, 9);
    Ok(Indicators{rsi: latest_rsi,
            sma_50: latest_sma_50,
            sma_200:latest_sma_200,
            macd: *macd_line.last().unwrap(),
            macd_signal: *macd_signal.last().unwrap(),
            macd_histogram: *macd_histogram.last().unwrap(),
            bb_lower: latest_bb.lower,
            bb_middle:latest_bb.average,
            bb_upper:latest_bb.upper,
    last_close,})
}

#[cfg(test)]
mod tests {
    use super::*;

    // Wilder's RSI(14) on the StockCharts reference series: 70.46 on day 15.
    #[tokio::test]
    async fn rsi_is_wilder() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE stocks(id INTEGER PRIMARY KEY, symbol TEXT, name TEXT)").execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE prices(id INTEGER PRIMARY KEY, stock_id INTEGER, date TEXT, open REAL, high REAL, low REAL, close REAL, volume INTEGER)").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO stocks (id, symbol) VALUES (1, 'X')").execute(&pool).await.unwrap();
        let closes = [44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28];
        for (i, c) in closes.iter().enumerate() {
            crate::db::insert_price(&pool, 1, &format!("2024-01-{:02}", i + 1), *c, *c, *c, *c, 0).await.unwrap();
        }
        let ind = compute(&pool, "X", None).await.unwrap();
        assert!((ind.rsi - 70.46).abs() < 0.01, "rsi = {}", ind.rsi);
    }
}
