pub mod old_commands;

use anyhow::anyhow;
use chrono::NaiveDate;
use scraper::{Element, Html, Selector};

pub async fn get_comic(comic_date: NaiveDate) -> anyhow::Result<String> {
    // Format current date
    let time = comic_date.format("%Y/%m/%d");

    // Get html data of garfield data for current day
    let text = reqwest::get(format!("https://www.gocomics.com/garfield/{}", time))
        .await?
        .error_for_status()?
        .text()
        .await?;

    // Set up parsing data structures
    let html_data = Html::parse_document(&text);
    let selector =
        Selector::parse("div").map_err(|_| anyhow!("Cannot parse 'div' from the returned html"))?;

    // Parse comic image url from html data
    let comic_url = html_data
        .select(&selector)
        .into_iter()
        .filter(|element| {
            // Remove everything other than the comic
            element.has_class(
                &("comic".into()),
                scraper::CaseSensitivity::AsciiCaseInsensitive,
            )
        })
        .map(|element| {
            element
                .attr("data-image")
                .ok_or(anyhow!("Cannot parse 'data-image' from comic strip"))
        })
        .next()
        .ok_or(anyhow!("Could not parse comic image url from html data"))??;

    Ok(comic_url.to_owned())
}
