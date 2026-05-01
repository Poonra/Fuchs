use crate::db;
use crate::ingest;
use crate::indicators::compute;


pub async fn run(pool: &sqlx::SqlitePool){

    let args: Vec<String>=  std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("add") => {
            match args.get(2).map(|s| s.as_str()) {
                Some(symbol) =>{
                    db::add_stock(pool,symbol).await.expect("Failed to add stock");
                 println!("Added stock {}", symbol)}
                None => println!("Usage: fuchs add <SYMBOL>"),
            }
        },
        Some("list")=> {
            let stocks = db::list_stocks(pool).await.expect("failed to load stocks");
            for stock in &stocks {
                println!("{} - {}", stock.id, stock.symbol);

            }
        }
        Some("sync") => {
            let stocks = db::list_stocks(pool).await.expect("failed to load stocks");
            for stock in &stocks {
                ingest::sync_stock(pool, &stock.symbol).await.expect("Failed to sync stock");
            }
            }

        Some("score") => {
            match args.get(2).map(|s| s.as_str()) {
                Some(symbol) => {
                    let result = compute(pool, symbol).await
                        .expect("Failed to compute indicators");
                    println!("Indicators for {} : ", symbol);
                    println!("RSI(14): {:.2}", result.rsi);
                }
                None => println!("Usage: fuchs score <SYMBOL>"),
            }
        }
        None => println!("Usage: fuchs <add|list|sync>"),
        _ => println!("Invalid argument"),


    }

}