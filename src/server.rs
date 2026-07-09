use axum::{http::StatusCode, routing::get, Router,extract::State,Json};
use axum::response::{IntoResponse, Response};
use sqlx::SqlitePool;
use crate::db;
use crate::models::Stock;
use axum::extract::Path;
use crate::indicators::{compute, Indicators};
use crate::scoring::{score_technical, ScoreBreakdown, score_fundamental, score_combined,CombinedScore};
use crate::models::Price;
use tower_http::services::ServeDir;
use std::time::Duration;
use crate::ingest;



pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", self.0)).into_response()
        }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub async fn serve(pool:SqlitePool) -> anyhow::Result<()> { // register all routes here!!!

    //spawn scheduler endless loop

    let scheduler_pool = pool.clone();
    tokio::spawn(async move {
        run_scheduler(scheduler_pool).await;
    });
    let app = Router::new()
        .route("/api/stocks", get(list_stocks))
        .route("/api/stocks/{symbol}/prices", get(get_prices))
        .route("/api/stocks/{symbol}/score", get(get_score))
        .fallback_service(ServeDir::new("static"))

        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on 127.0.0.1:8080");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn run_scheduler(pool:SqlitePool){
    let interval_secs = 300; //5 min
    let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));

    ticker.tick().await; //sync already when runs up so no need sync

    loop{
        ticker.tick().await;
        println!("[Scheduler] tick: start sync");

        let stocks = match db::list_stocks(&pool).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[scheduler] failed to list stocks: {}", e);
                continue;
            }
        };

        for stock in &stocks {
            match ingest::sync_stock(&pool, &stock.symbol).await {
                Ok(_) => {},
                Err(e) => eprintln!("[scheduler] {} sync failed: {}", stock.symbol, e),
            }
        }

        println!("[Scheduler] sync complete {} stocks",stocks.len());
    }
}



async fn list_stocks(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Stock>>, AppError>{
    let stocks = db::list_stocks(&pool).await?;
    Ok(Json(stocks))
}

#[derive(serde::Serialize)]
struct ScoreResponse {
    symbol: String,
    indicators: Indicators,
    combined: CombinedScore,
}
async fn get_score(
    State(pool): State<SqlitePool>,
    Path(symbol): Path<String>,
) -> Result<Json<ScoreResponse>, AppError> {
    let symbol = symbol.to_uppercase();
    let indicators = compute(&pool, &symbol, None).await?;
    let tech_breakdown = score_technical(&indicators);

    let stock = db::get_stock_by_symbol(&pool, &symbol).await?
        .ok_or_else(|| anyhow::anyhow!("stock {} not found", symbol))?;
    let fundamentals = db::get_latest_fundamentals(&pool, stock.id).await?;
    let fund_breakdown = fundamentals.as_ref().map(score_fundamental);

    let combined = score_combined(tech_breakdown, fund_breakdown);

    let today = time::OffsetDateTime::now_utc().date().to_string();
    if let Err(e) = db::insert_score_history(
        &pool, stock.id, &today,
        combined.technical.total,
        combined.fundamental.as_ref().map(|f| f.total),
        combined.total,
    ).await {
        eprintln!("Failed to insert score for {}: {}", stock.symbol, e);

}
    Ok(Json(ScoreResponse {
        symbol,
        indicators,
        combined,
    }))
}

async fn get_prices(
    State(pool): State<SqlitePool>,
    Path(symbol): Path<String>,
) -> Result<Json<Vec<Price>>, AppError> {
    let symbol = symbol.to_uppercase();
    let stock = db::get_stock_by_symbol(&pool, &symbol)
        .await?
        .ok_or_else(|| anyhow::anyhow!("stock {} not found", symbol))?;
    let prices = db::get_prices(&pool, stock.id, None).await?;
    Ok(Json(prices))
}