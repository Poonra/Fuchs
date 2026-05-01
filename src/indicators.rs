
use sqlx::SqlitePool;
use crate::db::{get_prices, get_stock_by_symbol};
use anyhow::bail;
use ta::Next;
use ta::indicators::RelativeStrengthIndex;

#[derive(Debug)]
pub struct Indicators {
    pub rsi: f64,
}

pub async fn compute(pool: &SqlitePool,symbol: &str) -> anyhow::Result<Indicators>{
    let stock = get_stock_by_symbol(pool, symbol).await?
        .ok_or_else(|| anyhow::anyhow!("Stock {} not found", symbol))?;
    let prices = get_prices(pool, stock.id).await?;

    if prices.is_empty() {
            anyhow::bail!("No prices found for {}. Run `fuchs sync` first.", symbol);
    }
    let mut rsi = RelativeStrengthIndex::new(14).unwrap();
    let mut latest_rsi = 0.0;

    for price in &prices {
        latest_rsi = rsi.next(price.close);
    }
    Ok(Indicators{rsi: latest_rsi})
}