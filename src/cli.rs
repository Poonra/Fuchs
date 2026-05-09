use crate::db;
use crate::ingest;
use crate::indicators::compute;
use crate::scoring;
use crate::server;
use crate::fundamentals;

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

            let mode = args.get(2).map(|s| s.as_str());

            match mode {
                Some("--fundamentals") => {
                    for stock in &stocks {
                        if let Err(e) = fundamentals::sync_fundamentals(pool, &stock.symbol).await {
                            eprintln!("Failed to sync fundamentals for {}: {}", stock.symbol, e);
                        }
                        // Alpha Vantage free tier: 5 requests/minute. Sleep 13s between calls.
                        tokio::time::sleep(std::time::Duration::from_secs(13)).await;
                    }
                }
                Some("--all") => {
                    for stock in &stocks {
                        if let Err(e) = ingest::sync_stock(pool, &stock.symbol).await {
                            eprintln!("Failed to sync prices for {}: {}", stock.symbol, e);
                        }
                        if let Err(e) = fundamentals::sync_fundamentals(pool, &stock.symbol).await {
                            eprintln!("Failed to sync fundamentals for {}: {}", stock.symbol, e);
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(13)).await;
                    }
                }
                None => {
                    // default: prices only
                    for stock in &stocks {
                        ingest::sync_stock(pool, &stock.symbol).await
                            .expect("Failed to sync stock");
                    }
                }
                Some(other) => {
                    println!("Unknown sync flag: {}. Use --fundamentals or --all.", other);
                }
            }
        }

        

        Some("score") => {
            match args.get(2).map(|s| s.as_str()) {
                Some(symbol) => {
                    let symbol = symbol.to_uppercase();

                    let as_of: Option<String> = args.iter()
                        .position(|a| a == "--date")
                        .and_then(|i| args.get(i+1))
                        .cloned();

                    // Technical
                    let indicators = compute(pool, &symbol, as_of.as_deref()).await
                        .expect("Failed to compute indicators");
                    let tech_breakdown = scoring::score_technical(&indicators);

                    // Fundamental (optional — may be absent)
                    let stock = db::get_stock_by_symbol(pool, &symbol).await
                        .expect("DB error")
                        .expect("Stock not in watchlist");
                    let fundamentals = db::get_latest_fundamentals(pool, stock.id).await
                        .expect("DB error");
                    let fund_breakdown = fundamentals.as_ref().map(scoring::score_fundamental);

                    // Combined
                    let combined = scoring::score_combined(
                        tech_breakdown,
                        fund_breakdown,
                    );

                    // Output
                    println!("Score for {}", symbol);
                    println!("  Last close:      {:.2}", indicators.last_close);
                    println!("  RSI(14):         {:.2}", indicators.rsi);
                    println!("  SMA(50):         {:.2}", indicators.sma_50);
                    println!("  SMA(200):        {:.2}", indicators.sma_200);
                    println!("  MACD:            {:.2}", indicators.macd);
                    println!("  BB:              {:.2} / {:.2} / {:.2}",
                             indicators.bb_lower, indicators.bb_middle, indicators.bb_upper);

                    println!("\nTechnical: {}/100", combined.technical.total);
                    for hit in &combined.technical.hits {
                        let sign = if hit.points >= 0 { "+" } else { "" };
                        println!("  {}{}  {}", sign, hit.points, hit.rule);
                    }

                    match &combined.fundamental {
                        Some(f) => {
                            println!("\nFundamental: {}/100", f.total);
                            for hit in &f.hits {
                                let sign = if hit.points >= 0 { "+" } else { "" };
                                println!("  {}{}  {}", sign, hit.points, hit.rule);
                            }
                        }
                        None => {
                            println!("\nFundamental: no data (run `fuchs sync --fundamentals`)");
                        }
                    }

                    println!("\nCOMBINED: {}/100", combined.total);
                }
                None => println!("Usage: fuchs score <SYMBOL>"),
            }
        }

        Some("serve") => {
            server::serve(pool.clone()).await.expect("server crashed");
        }

        None => println!("Usage: fuchs <add|list|sync [--fundamentals|--all]|score|serve>"),
        _ => println!("Invalid argument"),


    }

}