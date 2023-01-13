use std::time::Instant;
use serenity::model::channel::Message;
use serenity::client::Context;
use serenity::framework::standard::macros::*;
use serenity::framework::standard::CommandResult;

#[command]
///`e.ping`
///Find if the bot is dead.
///pong
#[aliases(pong)]
pub async fn ping(
    ctx: &Context,
    msg: &Message
) -> CommandResult {
    let time : Instant = Instant::now();
    let mut reply = msg.reply_ping(&ctx, "🔫 Pong! 🔫 ...").await?;

    reply.edit(
        &ctx,
        |m| m.content(format!(
            "🔫 Pong! 🔫, {}ms",
            time.elapsed().as_millis()
        ))
    ).await?;

    Ok(())
}
