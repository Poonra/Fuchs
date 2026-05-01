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
                    println!("  Last close:      {:.2}", result.last_close);
                    println!("  RSI(14):         {:.2}", result.rsi);
                    println!("  SMA(50):         {:.2}", result.sma_50);
                    println!("  SMA(200):        {:.2}", result.sma_200);
                    println!("  MACD:            {:.2}", result.macd);
                    println!("  MACD signal:     {:.2}", result.macd_signal);
                    println!("  MACD histogram:  {:.2}", result.macd_histogram);
                    println!("  BB lower:        {:.2}", result.bb_lower);
                    println!("  BB middle:       {:.2}", result.bb_middle);
                    println!("  BB upper:        {:.2}", result.bb_upper);
                }
                None => println!("Usage: fuchs score <SYMBOL>"),
            }
        }
        None => println!("Usage: fuchs <add|list|sync>"),
        _ => println!("Invalid argument"),


    }

}