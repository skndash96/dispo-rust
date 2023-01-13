use serenity::client::Context;
use serenity::framework::standard::{
    Args,
    CommandResult,
    macros::*
};
use serenity::model::channel::Message;
use serenity::futures::stream::StreamExt;

use mongodb::bson::{
    doc,
    document::Document as BsonDoc
};
use mongodb::options::FindOptions;

use crate::utils::capitalize;
use crate::models::Players;
use crate::constants::EMBED_COLOR;

#[command]
///`e.rank <coins/xp/strike/wins/orange/highscore/totalscore/wickets/ducks>`
///Checkout global leaderboard on various fields.
///lb
#[aliases(lb)]
#[bucket = "secondary"]
#[min_args(1)]
pub async fn rank(
    ctx: &Context,
    msg: &Message,
    mut args: Args
) -> CommandResult {
    let field = args.single::<String>()?;

    let field = match field.to_lowercase().as_str() {
        "xp" => Ok("xp"),
        "coins" => Ok("cc"),
        /*^alias*/"cc" => Ok("cc"),
        "wins" => Ok("wins"),
        "strike" => Ok("strikeRate"),
        "highscore" => Ok("highScore"),
        "totalscore" => Ok("totalScore"),
        "orange" => Ok("orangeCaps"),
        "wickets" => Ok("wickets"),
        "ducks" => Ok("ducksTaken"),
        _other => Err("syntax")
    }?;
    let d_field : String = capitalize(&field);

    let pcoll = {
        let data = ctx.data.read().await;
        data.get::<Players>()
            .ok_or("Failed to retrieve Player Collection from context.")
            ?.clone()
    };

    let mut query = BsonDoc::new();
    query.insert(field, doc! {
        "$exists": true
    });
    let query = query;

    let mut show = BsonDoc::new();
    show.insert("_id", 1);
    show.insert(field, 1);
    let show = show;

    let mut sort = BsonDoc::new();
    sort.insert(field, -1);
    let sort = sort;

    let mut cursor = pcoll.find(query, FindOptions::builder()
        .projection(show)
        .sort(sort)
        .limit(15)
        .build()
    ).await?;

    let mut i = 1;
    let mut rank_text : String = String::from("🔥 Top 10 🔥\n");

    while let Some(player) = cursor.next().await {
        if i > 10 {
            break;
        }

        if player.is_err() {
            eprintln!(
                "Error in rank while moving cursor : {:?}",
                 player.unwrap_err().to_string()
            );
            continue;
        }

        let player = player.unwrap();

        let id = player._id
            .parse::<u64>()
            .unwrap_or(0);
        
        let num : u64 = match field {
            "xp" => player.xp.unwrap() as u64,
            "cc" => player.cc.unwrap() as u64,
            "wins" => player.wins.unwrap() as u64,
            "strikeRate" => player.strikeRate.unwrap() as u64,
            "highScore" => player.highScore.unwrap() as u64,
            "totalScore" => player.totalScore.unwrap() as u64,
            "orangeCaps" => player.orangeCaps.unwrap() as u64 as u64,
            "wickets" => player.wickets.unwrap() as u64,
            "ducksTaken" => player.ducksTaken.unwrap() as u64,
            _other => 0_u64
        };
        let name = ctx.http
            .get_user(id)
            .await
            .map_or(
                String::from("Anonymous"),
                |u| u.name
            );
        
        rank_text.push_str((
            if i < 4 {
                format!(
                    "{}  |  **{}** `{:.0}`\n\n",
                    match i {
                        1 => "🥇",
                        2 => "🥈",
                        3 => "🥉",
                        _ => ""
                    }, name, num
                )
            } else {
                format!(
                    "{}  |  `{:.0}` **{}**\n",
                    i, num, name
                )
            }
        ).as_str());
            
        i += 1;
    }

    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e
            .title(
                format!("{} Leaderboard", d_field
            ))
            .description(rank_text)
            .color(EMBED_COLOR)
            .footer(|f| f.text(
                "Standing in the Hall Of Fame."
            ))
        })
    }).await?;

    Ok(())
}