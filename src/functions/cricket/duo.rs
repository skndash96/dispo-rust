use std::time::Duration;
use tokio::time::sleep;

use serenity::model::user::User;
use serenity::model::application::component::ButtonStyle;
use serenity::model::channel::Message;
use serenity::client::Context;

use super::start::start_match;
use crate::constants::TIMEOUT;
use crate::models::HcOptions;
use crate::utils::get_emoji;

pub async fn set_match(
    ctx: &Context,
    msg: &Message,
    u: &User,
    t: &User,
    options: HcOptions
) -> Result<(), String> {
    let channel_id = msg.channel_id;

    let mut talk_msg = channel_id.send_message(&ctx, |m| {
        m.content(format!(
            "<@{}>, Do you want to play handcricket with **{}**?",
            t.id, u.name
        ))
        .reference_message(msg)
        .components(|c| c.create_action_row(|r|
            r
            .create_button(|b| {
                b.style(ButtonStyle::Success)
                .label("yes")
                .custom_id("y")
            })
            .create_button(|b| {
                b.style(ButtonStyle::Danger)
                .label("no")
                .custom_id("n")
            })
        ))
    }).await
    .map_err(|e| e.to_string())?;

    let will_res = talk_msg
        .await_component_interaction(&ctx)
        .collect_limit(1)
        .author_id(t.id)
        .message_id(talk_msg.id)
        .timeout(Duration::new(TIMEOUT, 0))
        .await;

    let will = match will_res {
        Some(res) => {
             match res.data.custom_id.as_str() {
                "y" => true,
                _ => false
            }
        },
        None => false
    };

    if !will {
        msg.reply(&ctx, format!(
            "{} failed to give a positive response, match cancelled.",
            t.name
        )).await
        .map_err(|e| e.to_string())?;

        talk_msg.delete(&ctx)
            .await
            .map_err(|e| e.to_string())?;
    
        return Ok(());
    }

    //TODO: Toss based on tossMulti
    let w = &u;
    let l = &t;

    let toss_em = get_emoji(&ctx, "toss")
        .await
        .unwrap_or(String::new());

    talk_msg.edit(&ctx, |m| {
        m.content(format!(
            "Rolling the toss... {}",
            toss_em
        )).components(|c| c)
    }).await
    .map_err(|e| e.to_string())?;

    sleep(Duration::new(5, 0)).await;

    talk_msg.edit(&ctx, |m| {
        m.content(format!(
            "<@{}>, you won the toss, what do you pick?",
            w.id
        ))
        .components(|c| c.create_action_row(|r|
            r
            .create_button(|b| {
                b.label("Bat")
                .style(ButtonStyle::Primary)
                .custom_id("bat")
            })
            .create_button(|b| {
                b.label("Bowl")
                .style(ButtonStyle::Primary)
                .custom_id("bowl")
            })
        ))
    }).await
    .map_err(|e| e.to_string())?;

    let choice_res = talk_msg
        .await_component_interaction(&ctx)
        .collect_limit(1)
        .author_id(w.id)
        .message_id(talk_msg.id)
        .timeout(Duration::new(TIMEOUT, 0))
        .await;

    let bat;
    let bowl;

    if let Some(choice) = choice_res {
        let [_bat, _bowl] = match choice.data.custom_id.as_str() {
            "bat" => [w, l],
            "bowl" => [l, w],
            _ => {
                eprintln!("Cricket choice response recieved neither bat nor bowl.");
                [l, w]
            }
        };

        bat = _bat;
        bowl = _bowl;
    } else {
        msg.reply(&ctx, format!(
            "{} failed to give a response, match cancelled.",
            w.name
        )).await
        .map_err(|e| e.to_string())?;

        talk_msg.delete(&ctx)
            .await
            .map_err(|e| e.to_string())?;

        return Ok(());
    }

    talk_msg.edit(&ctx, |m| {
        m.content("Match starts, move to DMs players!")
        .components(|c| c)
    }).await
    .map_err(|e| e.to_string())?;

    start_match(
        &ctx,
        channel_id,
        vec![&bat],
        vec![&bowl],
        &options,
        true
    ).await?;

    Ok(())
}
