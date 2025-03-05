use std::sync::Arc;

use anyhow::anyhow;
use chrono::NaiveDate;
use garfield_bot::{
    get_comic,
    old_commands::{Garfield, Help, Pinger},
};
use poise::CreateReply;
use serenity::{all::CreateEmbed, prelude::*};

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Gets the garfield comic strip at the given date
#[poise::command(slash_command, prefix_command)]
async fn garfield_at(
    ctx: Context<'_>,
    #[description = "year"] year: i32,
    #[description = "month"]
    #[min = 1]
    #[max = 12]
    month: u8,
    #[description = "day"]
    #[min = 1]
    #[max = 31]
    day: u8,
) -> Result<(), Error> {
    let time = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .ok_or(anyhow!("Invalid date '{year}-{month}-{day}'!"))?;

    garfield_message(ctx, time).await
}

/// Gets the garfield comic strip for today
#[poise::command(slash_command, prefix_command)]
async fn garfield(ctx: Context<'_>) -> Result<(), Error> {
    garfield_message(ctx, chrono::Local::now().date_naive()).await
}

async fn garfield_message(ctx: Context<'_>, time: NaiveDate) -> Result<(), Error> {
    let comic_date = time.format("%B %d, %Y");

    ctx.send(
        CreateReply::default()
            .content(format!("Garfield: {comic_date}"))
            .embed(CreateEmbed::new().image(get_comic(time).await?)),
    )
    .await?;

    Ok(())
}

/// Sends a butter to register or deregister slash commands
#[poise::command(prefix_command)]
async fn register_commands(ctx: Context<'_>) -> Result<(), Error> {
    if ctx.author().id != 1192519637448011827 {
        Err(anyhow!("Not bot author"))?;
    }

    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}

/// Gets the garfield comic strip for today
#[poise::command(slash_command, prefix_command)]
async fn help(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Help:\n/garfield - to get the garfield commic strip of the day\n/garfield_at <year month day> - to get the garfield commic strip for the given date").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    let token = std::fs::read_to_string(".key").expect("Cannot read token");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            // Commands
            commands: vec![garfield(), garfield_at(), register_commands(), help()],
            // Preix options
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(3600),
                ))),
                case_insensitive_commands: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Pinger)
        .event_handler(Garfield)
        .event_handler(Help)
        .framework(framework)
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
