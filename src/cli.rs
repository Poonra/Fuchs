use crate::db;
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
        Some("sync") => println!("Sync"),
        None => println!("Usage: fuchs <add|list|sync>"),
        _ => println!("Invalid argument"),
    }

}