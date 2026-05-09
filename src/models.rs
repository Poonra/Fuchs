#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Stock{
    pub id: i64,
    pub symbol: String,
    pub name: Option<String>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Price{
    pub id: i64,
    pub stock_id: i64,
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Fundamentals {
    pub id: i64,
    pub stock_id: i64,
    pub as_of_date: String,
    pub sector: Option<String>,
    pub market_cap: Option<f64>,
    pub pe_ratio: Option<f64>,
    pub peg_ratio: Option<f64>,
    pub pb_ratio: Option<f64>,
    pub eps: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub profit_margin: Option<f64>,
    pub roe: Option<f64>,
    pub debt_to_equity: Option<f64>,
    pub revenue_ttm: Option<f64>,
    pub quarterly_revenue_growth: Option<f64>,
    pub quarterly_earnings_growth: Option<f64>,
}
