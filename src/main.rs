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

    dotenvy::from_path("C:/Users/poonr/Desktop/Project/Fuchs/.env").ok();
    dotenvy::dotenv().ok(); // fallback: load from cwd if above path missingh
    let pool = db::init_db().await.expect("Failed to init DB");


    cli::run(&pool).await;

    
}