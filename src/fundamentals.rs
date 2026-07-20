use serde::Deserialize;
use sqlx::SqlitePool;
use crate::db;
use time::OffsetDateTime;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OverviewResponse {
    pub symbol: Option<String>,
    pub sector: Option<String>,
    pub market_capitalization: Option<String>,
    #[serde(rename = "PERatio")]
    pub pe_ratio: Option<String>,
    #[serde(rename = "PEGRatio")]
    pub peg_ratio: Option<String>,
    pub price_to_book_ratio: Option<String>,
    #[serde(rename = "EPS")]
    pub eps: Option<String>,
    pub dividend_yield: Option<String>,
    pub profit_margin: Option<String>,
    #[serde(rename = "ReturnOnEquityTTM")]
    pub return_on_equity_ttm: Option<String>,
    pub debt_to_equity_ratio: Option<String>,
    #[serde(rename = "RevenueTTM")]
    pub revenue_ttm: Option<String>,
    #[serde(rename = "QuarterlyRevenueGrowthYOY")]
    pub quarterly_revenue_growth_yoy: Option<String>,
    #[serde(rename = "QuarterlyEarningsGrowthYOY")]
    pub quarterly_earnings_growth_yoy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EarningsResponse {
    #[serde(rename = "quarterlyEarnings")]
    pub quarterly_earnings: Option<Vec<QuarterlyEarning>>,
}

#[derive(Debug, Deserialize)]
pub struct QuarterlyEarning {
    #[serde(rename = "fiscalDateEnding")]
    pub fiscal_date_ending: String,
    #[serde(rename = "reportedEPS")]
    pub reported_eps: Option<String>,
    #[serde(rename = "estimatedEPS")]
    pub estimated_eps: Option<String>,
    #[serde(rename = "surprisePercentage")]
    pub surprise_percentage: Option<String>,
}

fn parse_f64(s: &Option<String>) -> Option<f64> {
    match s {
        Some(v) if v != "None" && v != "-" && !v.is_empty() => v.parse::<f64>().ok(),
        _ => None,
    }
}

const ALPHA_VANTAGE_BASE: &str = "https://www.alphavantage.co/query";

pub async fn sync_fundamentals(pool: &SqlitePool, symbol: &str) -> anyhow::Result<()> {
    let api_key = std::env::var("ALPHA_VANTAGE_API_KEY")
        .map_err(|_| anyhow::anyhow!("ALPHA_VANTAGE_API_KEY not set. Add it to .env"))?;

    let stock = match db::get_stock_by_symbol(pool, symbol).await? {
        Some(s) => s,
        None => {
            println!("Stock {} not in watchlist. Add it first.", symbol);
            return Ok(());
        }
    };

    let url = format!(
        "{}?function=OVERVIEW&symbol={}&apikey={}",
        ALPHA_VANTAGE_BASE, symbol, api_key
    );

    let response = reqwest::get(&url).await?;
    let body = response.text().await?;

    // Alpha Vantage returns 200 OK with an error message in the body when rate-limited
    // or when the symbol is invalid. Detect that before trying to parse.
    if body.contains("\"Note\"") || body.contains("\"Information\"") {
        anyhow::bail!("Alpha Vantage API limit or error: {}", body);
    }

    let overview: OverviewResponse = serde_json::from_str(&body)?;

    if overview.symbol.is_none() {
        anyhow::bail!("Empty response for {}. Symbol may be invalid.", symbol);
    }

    let now = OffsetDateTime::now_utc();
    let as_of_date = format!("{}-{:02}-{:02}", now.year(), now.month() as u8, now.day());

    db::insert_fundamentals(
        pool,
        stock.id,
        &as_of_date,
        overview.sector.as_deref(),
        parse_f64(&overview.market_capitalization),
        parse_f64(&overview.pe_ratio),
        parse_f64(&overview.peg_ratio),
        parse_f64(&overview.price_to_book_ratio),
        parse_f64(&overview.eps),
        parse_f64(&overview.dividend_yield),
        parse_f64(&overview.profit_margin),
        parse_f64(&overview.return_on_equity_ttm),
        parse_f64(&overview.debt_to_equity_ratio),
        parse_f64(&overview.revenue_ttm),
        parse_f64(&overview.quarterly_revenue_growth_yoy),
        parse_f64(&overview.quarterly_earnings_growth_yoy),
    ).await?;

    println!("Synced fundamentals for {}", symbol);
    Ok(())
}

pub async fn sync_earnings(pool: &SqlitePool, symbol: &str) -> anyhow::Result<()> {
    let api_key = std::env::var("ALPHA_VANTAGE_API_KEY")
        .map_err(|_| anyhow::anyhow!("ALPHA_VANTAGE_API_KEY not set. Add it to .env"))?;

    let stock = match db::get_stock_by_symbol(pool, symbol).await? {
        Some(s) => s,
        None => {
            println!("Stock {} not in watchlist. Add it first.", symbol);
            return Ok(());
        }
    };

    let url = format!(
        "{}?function=EARNINGS&symbol={}&apikey={}",
        ALPHA_VANTAGE_BASE, symbol, api_key
    );

    let response = reqwest::get(&url).await?;
    let body = response.text().await?;

    if body.contains("\"Note\"") || body.contains("\"Information\"") {
        anyhow::bail!("Alpha Vantage API limit or error: {}", body);
    }

    let earnings: EarningsResponse = serde_json::from_str(&body)?;
    let quarters = earnings.quarterly_earnings.unwrap_or_default();

    if quarters.is_empty() {
        anyhow::bail!("Empty earnings response for {}. Symbol may be invalid.", symbol);
    }

    // Last 4 quarters is enough for a beat/miss streak rule.
    for q in quarters.iter().take(4) {
        db::insert_earnings(
            pool,
            stock.id,
            &q.fiscal_date_ending,
            parse_f64(&q.reported_eps),
            parse_f64(&q.estimated_eps),
            parse_f64(&q.surprise_percentage),
        ).await?;
    }

    println!("Synced earnings for {}", symbol);
    Ok(())
}