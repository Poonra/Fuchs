use sqlx::SqlitePool;
use anyhow::Result;
use crate::db;
use sha2::{Sha256, Digest };
const FEED_URL: &str = "https://www.hoonsmart.com/feed";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";

pub async fn sync(pool: &SqlitePool) -> Result<()> {
    let source_id = db::ensure_source(pool, "hoonsmart", "rss", FEED_URL).await?;

    let body = reqwest::Client::new()
        .get(FEED_URL)
        .header("User-Agent", UA)
        .send().await?
        .error_for_status()?
        .bytes().await?;

    let feed = feed_rs::parser::parse(&body[..])?;
    let fetched_at = time::OffsetDateTime::now_utc().to_string();

    let (mut inserted, mut skipped) = (0, 0);
    for entry in &feed.entries {
        let Some(url) = entry.links.first().map(|l| l.href.as_str()) else {
            continue; // no URL = no dedup key, drop it
        };
        let hash = format!("{:x}", Sha256::digest(url.as_bytes()));

        let title = entry.title.as_ref()
            .map(|t| t.content.as_str())
            .unwrap_or("(no title)");

        let art_body = entry.content.as_ref().and_then(|c| c.body.as_deref())
            .or(entry.summary.as_ref().map(|s| s.content.as_str()));

        let published = entry.published.map(|d| d.to_rfc3339());

        if db::insert_article(pool, source_id, url, &hash, title,
                              art_body, published.as_deref(), &fetched_at).await? {
            inserted += 1;
        } else {
            skipped += 1;
        }
    }
    println!("news sync: {} new, {} duplicate", inserted, skipped);
    Ok(())
}

pub async fn list(pool: &SqlitePool) -> Result<()> {
    for (date, title) in db::list_articles(pool, 20).await? {
        println!("{}  {}", date, title);
    }
    Ok(())
}