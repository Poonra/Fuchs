mod cli;
mod db;
mod ingest;
mod models;
mod indicators;
mod scoring;
mod server;

#[tokio::main]
async fn main() {
    let pool = db::init_db().await.expect("Failed to init DB");


    cli::run(&pool).await;

    
}