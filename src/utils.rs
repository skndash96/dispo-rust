use std::env;
use std::boxed::Box;
use std::error::Error;
use std::collections::HashMap;
use std::fs;
use std::io::{self, prelude::*};

use serenity::Client;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::{
    DispatchError,
    macros::hook
};
use serenity::model::prelude::Message;
use serenity::futures::stream::StreamExt;

use mongodb::{
    Database,
    Collection
};
use mongodb::Client as MongoClient;
use mongodb::options::ClientOptions as MongoClientOptions;
use mongodb::bson::Document as BsonDocument;

use crate::models::{
    Player,
    Commands,
    CommandsInfo,
    Engaged,
    EngagedType,
    Emojis,
    EmojisType
};

pub async fn fix_status(
    ctx: &Context,
    ids: &Vec<String>
) -> Result<(), String> {
    let mut data = ctx.data.write().await;
    let engaged = data.get_mut::<Engaged>()
        .ok_or("Failed to get engaged users data from context")
        ?;

    (*engaged).drain_filter(|id| ids.contains(id));
    return Ok(());
}

pub fn check_status(
    engaged: &EngagedType,
    ids: &Vec<String>
) -> Result<(), String> {
    for id in ids.into_iter() {
        if engaged.contains(id) {
            return Err(format!(
                "status: {}",
                id
            ));
        }
    }
    return Ok(());
}

pub async fn get_player_data(
    db: &Collection<Player>,
    id : String
) -> Result<
    Player, 
    String
> {
    let mut query = BsonDocument::new();
    query.insert("_id", &id);
    let query = query;

    let mut cursor = db.find(query, None)
        .await
        .map_err(|e| e.to_string())?;
    let data = cursor.next().await;

    match data {
        Some(Ok(p)) => Ok(p),
        Some(Err(why)) => Err(why.to_string()),
        None => Err(format!(
            "data: {}",
            id
        ))
    }
}

pub async fn get_cmd_syntax(
    ctx: &Context,
    cmd: &str
) -> String {
    let cmd = &cmd.to_string();
    let cmds = {
        let data = ctx.data.read().await;
        data.get::<Commands>()
            .ok_or("Failed to get commands from context")
            //Potential Panic
            .unwrap()
            .clone()
    };

    if let Some(value) = cmds.get(cmd) {
        return value[0].to_owned();
    } else {
        let syn = format!("`e.help {}`", cmd);
        return syn;
    };
}

#[hook]
pub async fn after_command(
    ctx: &Context,
    msg: &Message,
    cmd: &str,
    err: CommandResult
) {
    if let Err(why) = err {
        let why = why.to_string();

        if why == "syntax" {
            let syntax = get_cmd_syntax(&ctx, &cmd).await;

            let _ = msg.reply(&ctx.http, format!(
                "Given command arguments are not valid, checkout the syntax:\n{}",
                syntax
            )).await;
        } else if why == "self_mention" {
            let _ = msg.reply(
                &ctx.http,
                "Wish you can mention yourself, but you can't."
            ).await;
        } else if why.starts_with("data:") {
            let id = &why[6..]; /*"data: XXXXXXXXXXXXX"*/

            let _ = msg.reply(&ctx.http, format!(
                "<@{}> is not a Dispoian yet. Type in `e.start` to get started.",
                id
            )).await;
        } else if why.starts_with("status:") {
            let id = &why[8..]; /*"status: XXXXXXXXXXXXX"*/
            let _ = msg.reply(&ctx.http, format!(
                "<@{}> is currently engaged in a game, try later.",
                id
            )).await;
        } else {
            eprintln!("ERROR in {}: {:?}", cmd, why);
        }
    }
}

#[hook]
pub async fn dispatch_error (
    ctx: &Context,
    msg: &Message,
    error: DispatchError,
    cmd: &str,
) {
    match error {
        DispatchError::Ratelimited (info) => {
            if info.is_first_try {
                let _ = msg.reply(ctx, format!(
                    "Hold your horse, try this again in {:.2} seconds.",
                    (info.as_millis() / 1000)
                )).await;
            }
        },
        DispatchError::NotEnoughArguments {..} => {
            let syntax = get_cmd_syntax(&ctx, &cmd).await;
            let _ = msg.reply(&ctx.http, format!(
                "Too less Arguments provided. Checkout syntax\n{}",
                syntax
            )).await;
        },
        DispatchError::TooManyArguments {..} => {
            let syntax = get_cmd_syntax(&ctx, &cmd).await;
            let _ = msg.reply(&ctx.http, format!(
                "Too many Arguments provided. Checkout syntax\n{}",
                syntax
            )).await;
        },
        e => {
            eprintln!("ERROR on_dispatch_error! {:?}", e);
            let _ = msg.reply(&ctx.http, format!(
                "Something went wrong while processing the command. Try again or report to the official server."
            )).await;
        }
    };
}

pub async fn get_emoji(
    ctx: &Context,
    name: &str
) -> Option<String> {
    let data = ctx.data.read().await;
    let emojis = data.get::<Emojis>();

    if let Some(map) = emojis {
        map.get(name).cloned()
    } else {
        eprintln!("Emojis data is None in context.");
        None
    }
}

pub async fn get_emojis(
    client: &Client
) -> Result<
    EmojisType,
    String
> {
    let http = &client.cache_and_http.http;
    let mut emojis : EmojisType = HashMap::new();

    let general = ["920880917566861332"].iter();
    // let decorEmojis = ["953616925009784882"];
    // let unoEmojis = ["980684246605758514", "981395381625704468"];

    for gid in general {
        let id = gid.parse::<u64>()
            .map_err(|e| e.to_string())
            ?;
        let res = http.get_emojis(id).await
            .map_err(|e| e.to_string())
            ?;
    
        for em in res {
            emojis.insert(
                em.name.clone(),
                format!(
                    "<:{}:{}>",
                    em.name,
                    em.id
                )
            );
        }
    }

    Ok(emojis)
}

pub fn get_commands() -> Result<
    CommandsInfo,
    io::Error> {
    let mut info_map : CommandsInfo = HashMap::new();

    let mut grps = fs::read_dir("./src/commands")?;

    while let Some(grp) = grps.next() {
        let grp = grp?.path();

        if !grp.is_dir() {
            continue;
        }

        let mut cmds = fs::read_dir(grp)?;
        while let Some(cmd) = cmds.next() {
            let cmd = cmd?.path();
            let cmd_name = cmd.file_name().unwrap()
                .to_str().unwrap();

            if cmd_name.ends_with("mod.rs") {
                continue;
            }

            let f = fs::File::open(&cmd)?;
            let mut reader = io::BufReader::new(f);

            let mut txt = String::new();
            let mut info = Vec::new();

            while 0_usize != reader.read_line(&mut txt)? {
                if txt.starts_with("///") {
                    txt.pop();
                    info.push(txt[3..].to_owned());
                }
                txt.clear();
            }

            info_map.insert(
                cmd_name[.. cmd_name.len() - 3]
                .to_owned(),
                info
            );
        }
    }

    Ok(info_map)
}

pub async fn make_db_connection() -> Result<Database, Box<dyn Error>> {
    let mongo_uri = env::var("MONGO_URI")?;
    let m_client_options = MongoClientOptions::parse(mongo_uri).await?;
    let m_client = MongoClient::with_options(m_client_options)?;

    Ok(m_client.database("myFirstDatabase"))
}

pub fn capitalize(i: &str) -> String {
    let mut c = i.chars();
    match c.next() {
        None => String::new(),
        Some(l) => l.to_uppercase().collect::<String>() + c.as_str()
    }
}
