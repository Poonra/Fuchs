use crate::db;
use crate::ingest;
use crate::indicators::compute;
use crate::scoring;
use crate::server;

pub async fn run(pool: &sqlx::SqlitePool){

    let args: Vec<String>=  std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("add") =>{
            match args.get(2).map(|s| s.as_str()) {
                Some(symbol) =>{
                    let symbol = symbol.to_uppercase();
                    db::add_stock(pool,&symbol).await.expect("Failed to add stock");
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
                    let symbol =symbol.to_uppercase();
                    
                    let as_of: Option<String> = args.iter()
                        .position(|a| a == "--date")
                        .and_then(|i| args.get(i+1))
                        .cloned();
                    let result = compute(pool, &symbol, as_of.as_deref()).await
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

                    let breakdown = scoring::score_technical(&result);
                    println!("\nTechnical score: {}/100; (50 is neutral)", breakdown.total);
                    println!("Breakdown:");
                    for hit in &breakdown.hits {
                        let sign = if hit.points >= 0 { "+" } else { "" };
                        println!("  {}{}  {}", sign, hit.points, hit.rule);
                    }
                }
                None => println!("Usage: fuchs score <SYMBOL>"),
            }
        }

        Some("serve") => {
            server::serve(pool.clone()).await.expect("server crashed");
        }

        None => println!("Usage: fuchs <add|list|sync|score|serve>"),
        _ => println!("Invalid argument"),


    }

}