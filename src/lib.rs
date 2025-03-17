pub mod old_commands;

use anyhow::anyhow;
use chrono::NaiveDate;
use scraper::{Html, Selector};

pub async fn get_comic(comic_date: NaiveDate) -> anyhow::Result<String> {
    // Format date as YYYY/MM/DD (GoComics uses this format for URLs)
    let time = comic_date.format("%Y/%m/%d").to_string();

    // Get HTML data of Garfield comic from GoComics for the specific date
    let text = reqwest::get(format!("https://www.gocomics.com/garfield/{}", time))
        .await?
        .text()
        .await?;

    // Set up parsing data structures
    let html_data = Html::parse_document(&text);
    
    // Use a selector to find the 'img' tag inside the 'picture' element with class 'item-comic-image'
    let selector = Selector::parse("picture.item-comic-image img").map_err(|_| anyhow!("Cannot parse 'img' selector"))?;

    // Attempt to find the image tag and extract its src attribute
    let comic_url = html_data
        .select(&selector)
        .next()
        .ok_or(anyhow!("Could not find comic image in HTML"))?
        .value()
        .attr("src")
        .ok_or(anyhow!("Could not extract 'src' attribute from comic image"))?;

    let full_url = format!("{}", comic_url); // Ensure the URL is properly formed with the full domain
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
