use std::time::Duration;

use super::start::start_match;

use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::user::User;
use serenity::model::id::EmojiId;

use crate::utils::get_emoji;
use crate::models::HcOptions;

pub async fn set_match(
    ctx: &Context,
    msg: &Message,
    u: &User,
    options: HcOptions
) -> Result<(), String> {
    let channel_id = msg.channel_id;

    let enter_em = {
        let em = get_emoji(
            &ctx,
            "enter"
        ).await
        .ok_or("Failed to get enter emoji")
        ?;

        let id = em[
            /*"<:enter:XXXXXXXXX>"*/
            8..em.len()-1
        ].parse::<u64>()
        .map_err(|e| format!(
            "Cannot parse enter emoji as u64: {}",
            e.to_string()
        ))?;

        Ok(EmojiId::from(id))
    }?;

    let talk_msg = channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e
            .title("Team Match")
            .description(format!(
                "React to join **{}**'s match",
                u.name
            ))
        })
        .reference_message(msg)
        .reactions([enter_em])
    })
    .await
    .map_err(|e| e.to_string())
    ?;

    let stream = talk_msg
        .await_reactions(&ctx)
        .message_id(talk_msg.id)
        .timeout(Duration::new(60, 0))
        .added(true)
        .removed(true)
        .build();

    //fidififkfkfjfkfjfjfjfjfjfjfjfjfjfjffjjccjfjcjcj
    while let r = stream.poll_next() {
        println!("{:?}", r);
    }

    Ok(())
}