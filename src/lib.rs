pub mod old_commands;

use anyhow::anyhow;
use chrono::NaiveDate;
use scraper::{Html, Selector};

pub async fn get_comic(comic_date: NaiveDate) -> anyhow::Result<String> {
    // Format current date
    let time = comic_date.format("%Y%m%d");

    // Get html data of garfield data for current day
    let text = reqwest::get(format!("https://www.mezzacotta.net/garfield/?date={}", time))
        .await?
        .text()
        .await?;

    // Set up parsing data structures
    let html_data = Html::parse_document(&text);
    let selector = Selector::parse("p img").map_err(|_| anyhow!("Cannot parse 'p img' from the returned HTML"))?;

    // Parse comic image url from html data
    let comic_url = html_data
        .select(&selector)
        .next()
        .ok_or(anyhow!("Could not find comic image in HTML"))?
        .value()
        .attr("src")
        .ok_or(anyhow!("Could not extract 'src' attribute from comic image"))?;

    let full_url = format!("https://www.mezzacotta.net{}", comic_url);
    Ok(full_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_comic() {
        let today = chrono::Utc::now().naive_utc().date();
        match get_comic(today).await {
            Ok(img_url) => println!("Latest comic image URL: {}", img_url),
            Err(e) => eprintln!("Error fetching comic: {}", e),
        }
    }
}