use anyhow::anyhow;
use chrono::NaiveDate;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use crate::get_comic;

pub struct Pinger;

#[async_trait]
impl EventHandler for Pinger {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.to_ascii_lowercase() != "!ping" {
            return;
        }

        if msg.author.bot {
            return;
        }

        send_msg(&msg, &ctx, "Pong!").await;
    }
}

pub struct Garfield;

#[async_trait]
impl EventHandler for Garfield {
    async fn message(&self, ctx: Context, msg: Message) {
        // Only respond to garfields
        if !msg.content.to_ascii_lowercase().starts_with("!garfield") {
            return;
        }

        if msg.author.bot {
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
                    send_msg(&msg, &ctx, format!("Unable to parse date: '{}'", err)).await;
                    return;
                }
            }
        };

        // Get the comic url
        let url = match get_comic(time).await {
            Ok(url) => url,
            Err(err) => {
                eprintln!("Comic Error: {}", err);
                send_msg(&msg, &ctx, format!("Commic Error: {}", err)).await;
                return;
            }
        };

        let comic_date = time.format("%B %d, %Y");
        let pretext = format!("Garfield: {comic_date}");

        // Send msg saying date beforehand (fancy)
        send_msg(&msg, &ctx, pretext).await;

        // Send the comic url.
        // Discord will fetch the image for us (yay!).
        send_msg(&msg, &ctx, url).await;
    }
}

async fn send_msg(msg: &Message, ctx: &Context, text: impl Into<String>) {
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

pub struct Help;

#[async_trait]
impl EventHandler for Help {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.to_ascii_lowercase() != "!help" {
            return;
        }

        if msg.author.bot {
            return;
        }

        let help_text = "Garfield help:\n\nSee '/help' for new slash commands!\n\n!ping - The bot will respond with a pong\n!garfield - The current garfield commic for today\n!garfield 2024-02-01 - The garfield strip at the given date\n!help - This message";

        if let Err(why) = msg.channel_id.say(&ctx.http, help_text).await {
            eprintln!("Error sending message: {why:?}");
        }
    }
}
