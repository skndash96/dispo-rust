use super::start::start_match;

use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::user::User;

use crate::models::HcOptions;

pub async fn set_match(
    ctx: &Context,
    msg: &Message,
    u: &User,
    options: HcOptions
) -> Result<(), String> {
    //fkfkfjfjfjdjdhhfyfhchfhchchchvyvychchchchchjcjcjcjfjf
    let channel_id = msg.channel_id;

    let talk_msg = channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e
            .title("Team Match")
            .description(format!(
                "React to join **{}**'s match",
                u.name
            ))
        })
        //.reactions([])
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}