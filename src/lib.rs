pub mod old_commands;

use anyhow::anyhow;
use chrono::NaiveDate;
use enum_iterator::Sequence;
use rand::seq::SliceRandom;
use scraper::{Element, Html, Selector};

/// Gets the link to the garfield comic (as text) for the given input date.
pub async fn get_comic(comic_date: NaiveDate) -> anyhow::Result<String> {
    // Find a comic that can be parsed
    let mut comic = Err(anyhow!(""));

    // Shuffle to ensure fair polling.
    let mut sources = enum_iterator::all::<Source>().collect::<Vec<Source>>();
    sources.shuffle(&mut rand::rng());

    for source in sources {
        if comic.is_ok() {
            break;
        }

        comic = source.get_comic(comic_date).await;

        // Debug printing
        if comic.is_err() {
            eprintln!("'{:?}': '{:#?}'", source, comic);
        }
    }
    comic
}

/// The sources to retrieve comics from.
#[derive(Sequence, Clone, Copy, Debug)]
enum Source {
    GoComic,
    ProductionCentralus,
    Jikos,
}

impl Source {
    async fn get_comic(self, comic_date: NaiveDate) -> anyhow::Result<String> {
        match self {
            Source::GoComic => go_comic(comic_date).await,
            Source::ProductionCentralus => production_centralus(comic_date).await,
            Source::Jikos => jikos(comic_date).await,
        }
    }
}

async fn go_comic(comic_date: NaiveDate) -> anyhow::Result<String> {
    // Format current date
    let time = comic_date.format("%Y/%m/%d");

    // Get html data of garfield data for current day
    let response = reqwest::get(format!("https://www.gocomics.com/garfield/{}", time))
        .await?
        .error_for_status()?;

    // If it is not a successful response then return
    if !response.status().is_success() {
        Err(anyhow!(
            "Commic is not available. Website Status: {}",
            response.status()
        ))?
    }

    let text = response.text().await?;

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

async fn production_centralus(comic_date: NaiveDate) -> anyhow::Result<String> {
    // Format current date
    let time = comic_date.format("%Y/%m/%d");

    // Get html data of garfield data for current day
    let response = reqwest::get(format!(
        "https://production.centralus.gocomics.com/garfield/{}",
        time
    ))
    .await?
    .error_for_status()?;

    // If it is not a successful response then return
    if !response.status().is_success() {
        Err(anyhow!(
            "Commic is not available. Website Status: {}",
            response.status()
        ))?
    }

    let text = response.text().await?;

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
                &("Comic_comic__image_strip__hPLFq".into()),
                scraper::CaseSensitivity::AsciiCaseInsensitive,
            )
        })
        .map(|element| {
            element
                .attr("src")
                .ok_or(anyhow!("Image Source from comic strip"))
        })
        .next()
        .ok_or(anyhow!("Could not parse comic image url from html data"))??;

    Ok(comic_url.to_owned())
}

async fn jikos(comic_date: NaiveDate) -> anyhow::Result<String> {
    Ok(format!(
        "https://picayune.uclick.com/comics/ga/{}/ga{}.gif",
        comic_date.format("%Y"),
        comic_date.format("%y%m%d")
    ))
}
