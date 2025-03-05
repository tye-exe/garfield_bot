use std::str::FromStr;

use anyhow::anyhow;
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use scraper::{Element, ElementRef, Html, Selector, selector::CssLocalName};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

async fn get_comic(comic_date: NaiveDate) -> anyhow::Result<String> {
    // Format current date
    let time = comic_date.format("%Y/%m/%d");

    // Get html data of garfield data for current day
    let text = reqwest::get(format!("https://www.gocomics.com/garfield/{}", time))
        .await?
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

struct Pinger;

#[async_trait]
impl EventHandler for Pinger {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content != "!ping" {
            return;
        }

        if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
            eprintln!("Error sending message: {why:?}");
        }
    }
}

struct Garfield;

#[async_trait]
impl EventHandler for Garfield {
    async fn message(&self, ctx: Context, msg: Message) {
        // Only respond to garfields
        if !msg.content.starts_with("!garfield") {
            return;
        }

        // Use local time if none given
        let time = if msg.content == "!garfield" {
            chrono::Local::now().date_naive()
        } else {
            // Attempt to parse given time
            match parse_date(&msg.content) {
                Ok(time) => time,
                Err(err) => {
                    send_msg(msg, ctx, format!("Unable to parse date: '{}'", err)).await;
                    return;
                }
            }
        };

        // Get the comic url
        let url = match get_comic(time).await {
            Ok(url) => url,
            Err(err) => {
                eprintln!("Comic Error: {}", err);
                return;
            }
        };

        let comic_date = time.format("%B %d, %Y");
        let pretext = format!("Garfield: {comic_date}");

        // Send msg saying date beforehand (fancy)
        if let Err(why) = msg.channel_id.say(&ctx.http, pretext).await {
            eprintln!("Error sending message: {why:?}");
        }

        // Send the comic url.
        // Discord will fetch the image for us (yay!).
        if let Err(why) = msg.channel_id.say(&ctx.http, url).await {
            eprintln!("Error sending message: {why:?}");
        }
    }
}

async fn send_msg(msg: Message, ctx: Context, text: impl Into<String>) {
    if let Err(why) = msg.channel_id.say(&ctx.http, text).await {
        eprintln!("Error sending message: {why:?}");
    }
}

fn parse_date(content: &str) -> anyhow::Result<NaiveDate> {
    let date_text: String = content.chars().skip(10).collect();

    let year: String = date_text.chars().take(4).collect();
    let month: String = date_text.chars().skip(5).take(2).collect();
    let day: String = date_text.chars().skip(8).take(2).collect();

    NaiveDate::from_ymd_opt(year.parse()?, month.parse()?, day.parse()?)
        .ok_or(anyhow!("Invalid date"))
}

struct Help;

#[async_trait]
impl EventHandler for Help {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content != "!help" {
            return;
        }

        let help_text = "Garfield help:\n!ping - The bot will respond with a pong\n!garfield - The current garfield commic for today\n!garfield <2024-02-01> - The garfield strip at the given date\n!help - This message";

        if let Err(why) = msg.channel_id.say(&ctx.http, help_text).await {
            eprintln!("Error sending message: {why:?}");
        }
    }
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    // let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let token = std::fs::read_to_string(".env").expect("Cannot read token");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Pinger)
        .event_handler(Garfield)
        .event_handler(Help)
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
