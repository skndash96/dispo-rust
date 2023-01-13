#![feature(result_option_inspect)]
#![feature(drain_filter)]

use std::env;
use dotenv;

use serenity::Client;
use serenity::client::{
    EventHandler,
    Context
};
use serenity::async_trait;
use serenity::framework::standard::StandardFramework;
use serenity::model::gateway::Ready;
use serenity::prelude::GatewayIntents;

mod constants;
mod commands;
mod utils;
mod models;
mod functions;

use models::{
    Player,
    Players,
    Commands,
    Engaged,
    Emojis
};

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Logged In as {}", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let commands = utils::get_commands()
       .expect("while mapping commands.");
    println!("Mapped {} commands.", commands.len());

    let framework = StandardFramework::new()
        .configure(|c| c
            .prefix("e.")
            .case_insensitivity(true)
        ) // set the bot's prefix to "e."
        .bucket("primary", |b| b.delay(5).time_span(30).limit(5)).await
        .bucket("secondary", |b| b.delay(10).time_span(30).limit(2)).await
        .bucket("tertiary", |b| b.delay(60)).await
        .on_dispatch_error(utils::dispatch_error)
        .after(utils::after_command)
        .group(&commands::general::GENERAL_GROUP)
        .group(&commands::cricket::CRICKET_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token not found in env.");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let db = utils::make_db_connection()
        .await
        .expect("making mongodb connection.");
    let players = db.collection::<Player>("players");

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<Commands>(commands)
        .type_map_insert::<Players>(players)
        .type_map_insert::<Engaged>(vec![])
        .await.expect("building client");

    let emojis = utils::get_emojis(&client)
        .await.expect("failed to get emojis");
    println!("Mapped {} emojis", emojis.len());

    {
        let mut data = client.data.write().await;
        data.insert::<Emojis>(emojis);
    };

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}