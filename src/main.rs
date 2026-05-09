mod cli;
mod db;
mod ingest;
mod models;
mod indicators;
mod scoring;
mod server;
mod fundamentals;

#[tokio::main]
async fn main() {

    dotenvy::dotenv().ok();
    let pool = db::init_db().await.expect("Failed to init DB");


    cli::run(&pool).await;

    
}