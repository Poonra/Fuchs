mod cli;
mod db;
mod ingest;
mod models;

#[tokio::main]
async fn main() {
    let pool = db::init_db().await.expect("Failed to init DB");
    println!("DB ready");

    cli::run(&pool).await;
}