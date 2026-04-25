use anyhow::anyhow;
use chrono::NaiveDate;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use scraper::{Html, Selector};

/// Gets the link to the heathcliff comic (as text) for the given input date.
pub async fn get_heathcliff(comic_date: NaiveDate) -> anyhow::Result<String> {
    let time = comic_date.format("%Y/%m/%d");

    let header_map: HeaderMap = [
        (
            USER_AGENT,
            "Mozilla/5.0 (X11; Linux x86_64; rv:149.0) Gecko/20100101 Firefox/149.0",
        ),
        (
            ACCEPT,
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        ),
        (ACCEPT_LANGUAGE, "en-US,en;q=0.9"),
    ]
    .into_iter()
    .map(|(head, val)| (head, HeaderValue::from_static(val)))
    .collect();

    let build = reqwest::Client::builder()
        .default_headers(header_map)
        .build()?;
    let response = build
        .get(format!("https://www.gocomics.com/heathcliff/{}", time))
        .send()
        .await?
        .error_for_status()?;

    let status = response.status();
    let is_success = status.is_success();
    let text = response.text().await?;

    // If it is not a successful response then return
    if !is_success {
        let collect: String = text.chars().take(1800).collect();
        Err(anyhow!(
            "Commic is not available.\nWebsite Status: {}\nWebsite says\n```html{}```",
            status,
            collect
        ))?
    }

    // Set up parsing data structures
    let html_data = Html::parse_document(&text);
    let selector = Selector::parse("button")
        .map_err(|_| anyhow!("Cannot parse 'button' from the returned html"))?;

    // Parse comic image url from html data
    let comic_url = html_data
        .select(&selector)
        .into_iter()
        .filter(|element| {
            if let Some(attr) = element.attr("aria-label")
                && attr == "Expand comic"
            {
                true
            } else {
                false
            }
        })
        .next()
        .map(|element| {
            element
                .select(&Selector::parse("img").unwrap())
                .next()
                .ok_or(anyhow!("Unable to get img for commic"))
                .map(|el| el.attr("src").ok_or(anyhow!("Unable to get src from img")))
                .flatten()
        })
        .ok_or(anyhow!("Could not parse comic image url from html data"))??;

    Ok(comic_url.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn heath() {
        let get_comic = get_comic(NaiveDate::from_ymd_opt(2026, 04, 24).unwrap())
            .await
            .inspect_err(|e| println!("{e}"))
            .unwrap();
        println!("{get_comic}");
        panic!("Ah");
    }
}
