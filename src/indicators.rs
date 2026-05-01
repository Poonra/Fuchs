use sqlx::SqlitePool;
use crate::db::{get_prices, get_stock_by_symbol};
use anyhow::bail;
use ta::Next;
use ta::indicators::{BollingerBands, BollingerBandsOutput, MovingAverageConvergenceDivergence, MovingAverageConvergenceDivergenceOutput, RelativeStrengthIndex, SimpleMovingAverage};

#[derive(Debug)]
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

pub async fn compute(pool: &SqlitePool,symbol: &str) -> anyhow::Result<Indicators>{
    let stock = get_stock_by_symbol(pool, symbol).await?
        .ok_or_else(|| anyhow::anyhow!("Stock {} not found", symbol))?;
    let prices = get_prices(pool, stock.id).await?;

    if prices.is_empty() {
            bail!("No prices found for {}. Run `fuchs sync` first.", symbol);
    }
    let mut rsi = RelativeStrengthIndex::new(14).unwrap();
    let mut latest_rsi = 0.0;
    let mut sma_50 = SimpleMovingAverage::new(50).unwrap();
    let mut latest_sma_50 = 0.0;
    let mut sma_200 = SimpleMovingAverage::new(200).unwrap();
    let mut latest_sma_200 = 0.0;
    let mut macd = MovingAverageConvergenceDivergence::new(12,26,9).unwrap();
    let mut latest_macd = MovingAverageConvergenceDivergenceOutput{
            macd:0.0,
            signal:0.0,
            histogram:0.0
    };
    let mut bb = BollingerBands::new(20,2.0).unwrap();
    let mut latest_bb = BollingerBandsOutput{
            lower:0.0,
            average:0.0,
            upper:0.0
    };
    let mut last_close = 0.0;

    for price in &prices {
        latest_rsi = rsi.next(price.close);
        latest_sma_50 = sma_50.next(price.close);
        latest_sma_200 = sma_200.next(price.close);
        latest_macd = macd.next(price.close);
        latest_bb = bb.next(price.close);
        last_close = price.close;

    }
    Ok(Indicators{rsi: latest_rsi,
            sma_50: latest_sma_50,
            sma_200:latest_sma_200,
            macd: latest_macd.macd,
            macd_signal: latest_macd.signal,
            macd_histogram:latest_macd.histogram,
            bb_lower: latest_bb.lower,
            bb_middle:latest_bb.average,
            bb_upper:latest_bb.upper,
    last_close,})
}