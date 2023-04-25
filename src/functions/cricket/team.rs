use std::time::Duration;
use tokio::time::sleep;

use rand;
use rand::prelude::*;

use serenity::model::prelude::component::ButtonStyle;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::{ReactionType, ChannelId};
use serenity::model::user::User;
use serenity::model::id::EmojiId;
use serenity::futures::StreamExt;

use crate::constants::{TIMEOUT, EMBED_COLOR};
use crate::functions::cricket::start::start_match;
use crate::utils::{
    get_emoji,
    fix_status
};
use crate::models::{
    HcOptions,
    HcPlayer,
    Extra
};

pub async fn set_match<'a>(
    ctx: &Context,
    msg: &Message,
    u: &User,
    options: HcOptions
) -> Result<(), String> {
    let channel_id = msg.channel_id;

    let toss_em = get_emoji(
        &ctx,
        "toss"
    ).await
    .ok_or("Failed to get toss emoji")
    ?;

    let enter_em_id = {
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

        Ok::<EmojiId, String>(
            EmojiId::from(id)
        )
    }?;
    let enter_em = ReactionType::Custom {
        name: Some(String::from("enter")),
        id: enter_em_id,
        animated: false
    };
    let cross_em = ReactionType::Unicode(String::from("❌"));

    let talk_msg = channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e
            .title("Team Match")
            .description(format!(
                "React to join **{}**'s handcricket match.",
                u.name
            ))
            .color(EMBED_COLOR)
        })
        .reference_message(msg)
        .reactions([
            enter_em.clone() /*clone cause used later*/,
            cross_em
        ])
    })
    .await
    .map_err(|e| e.to_string())
    ?;

    let mut stream = talk_msg
        .await_reactions(&ctx)
        .timeout(Duration::new(60, 0))
        .added(true)
        .removed(true)
        .build();

    let mut ps_id : Vec<String>= vec![];

    while let Some(react) = stream.next().await {
        let is_added = react.is_added();
        let react = react.as_inner_ref().clone();

        let em_name = match &react.emoji {
                ReactionType::Custom { name, .. } => name.clone().unwrap_or(String::new()),
                ReactionType::Unicode ( name ) => name.clone(),
                _ => String::new()
            };

        let from = match react.user(&ctx)
            .await {
                Ok(val) => val,
                Err(why) => {
                    eprintln!("Reaction's user recieved on team match message is error: {:?}", why);
                    react.delete(&ctx)
                        .await
                        .map_err(|e| e.to_string())
                        ?;
                    continue;
                }
            };

        if is_added
        && em_name == "❌"
        && from.id == u.id {
            fix_status(
                ctx,
                &ps_id[..],
                false
            ).await?;

            talk_msg.delete(&ctx)
                .await
                .map_err(|e| e.to_string())?;

            msg.reply(&ctx, format!(
                "Team Match cancelled by {}.",
                from.name
            )).await
            .map_err(|e| e.to_string())?;

            return Ok(());
        }

        if em_name != "enter" {
            continue;
        }

        if is_added {
            ps_id.push(from.id.to_string());

            if let Err(why) = fix_status(
                &ctx,
                &ps_id[ps_id.len()-1 ..],
                true
            ).await {
                if !why.starts_with("status") {
                    eprintln!("Fix status to true in a team match reaction failed: {:?}", &why);
                }

                react.delete(&ctx)
                .await
                .map_err(|e| e.to_string())
                ?;
            };
        } else if let Some(idx) = 
            ps_id.iter().position(
                |id| *id == from.id.to_string()
            ) {
            ps_id.remove(idx);
        }

        msg.channel_id.say(&ctx, format!(
            "**{}** {} the team match.",
            from.name,
            if is_added { "joined" } else { "left"}
        )).await
        .map_err(|e| e.to_string())
        ?;
    }

    stream.stop();

    let ps = talk_msg.reaction_users(
        &ctx,
        enter_em,
        None,
        None
    ).await
    .map_err(|e| e.to_string())
    ?;

    if ps.len() > 22 {
        eprintln!("A match cancelled due to players >22.");
        Err("A maximum of 22 players can play a team match at once. The count is too high! Try with a fewer members.")?;
    } else if ps.len() < 3 {
        Err("Aw poor, a minimum of 3 players are required to start a team match.")?;
    }

    let mut ps_hc : Vec<HcPlayer> = vec![];
    for i in 0..ps.len() {
        ps_hc.push(HcPlayer::U(
            &ps[i]
        ));
    }
    if ps_hc.len() % 2 == 1 {
        ps_hc.push(HcPlayer::E(
            Extra(None)
        ));
    }

    let num = {
        let mut rng = rand::thread_rng();
        rng.gen_range(
            0..ps.len()-2
            //2 consecutive players are chosen
            //one -1 to ensure no vector out of bounds
            //another -1 to ensure extra wicket doesn't get chosen
            //min 4(3 users) in ps, at the least range would be 0..2 i.e 0,1 is returned.
        )
    };

    let mut ps_idx : Vec<usize> = (0..ps_hc.len()).collect();

    let cap1 = &ps[num];
    let mut t1_idx : Vec<usize> = vec![];
    t1_idx.push(ps_idx.remove(num));

    let cap2 = &ps[num+1/*won't overflow, rand range's 0..len()-2*/];
    let mut t2_idx : Vec<usize> = vec![];
    t2_idx.push(ps_idx.remove(num+1));

    channel_id.send_message(&ctx, |m| {
        m.content(format!(
            "Let's start with this. We got,\n Leader 1: {}\n Leader 2: {}.",
            cap1.name,
            cap2.name
        ))
    }).await
    .map_err(|e| e.to_string())
    ?;

    sleep(Duration::new(3, 0)).await;

    let mut is1pick = true;
    loop {
        if ps_idx.len() == 0 {
            break;
        }

        pick(
            &ctx,
            &channel_id,
            if is1pick {&cap1} else {&cap2},
            &ps_hc,
            &mut ps_idx,
            if is1pick {&mut t1_idx} else {&mut t2_idx}
        ).await?;

        is1pick = !is1pick;
    }

    let mut toss_msg = channel_id.say(&ctx, format!(
        "Teaming is done, let's flip the toss... {}",
        toss_em
    )).await
    .map_err(|e| e.to_string())
    ?;

    sleep(Duration::new(3, 0)).await;

    let is1win : bool = rand::random();
    toss_msg.edit(&ctx, |m|
        m.content(format!(
            "<@{}>'s team won the toss, choose:",
            if is1win {cap1.id} else {cap2.id}
        ))
        .components(|c|
            c.create_action_row(|r|
                r.create_button(|b|
                    b
                    .style(ButtonStyle::Primary)
                    .label("Bat")
                    .custom_id("bat")
                ).create_button(|b|
                    b
                    .style(ButtonStyle::Primary)
                    .label("Bowl")
                    .custom_id("bowl")
                )
            )
        )
    ).await
    .map_err(|e| e.to_string())
    ?;

    let toss_res = toss_msg
        .await_component_interaction(&ctx)
        .author_id(
            if is1win {cap1.id} else {cap2.id}
        )
        .collect_limit(1)
        .message_id(toss_msg.id)
        .timeout(Duration::new(TIMEOUT, 0))
        .await;

    let iswinbat = if let Some(res) = toss_res {
        match res.data.custom_id.as_str() {
            "bat" => true,
            _/*"bowl"*/ => false
        }
    } else {
        toss_msg.reply(&ctx, format!(
            "{} failed to give a response, match cancelled.",
            if is1win {cap1.name.clone()} else {cap2.name.clone()}
        )).await
        .map_err(|e| e.to_string())
        ?;

        toss_msg.delete(&ctx)
            .await
            .map_err(|e| e.to_string())
            ?;

        return Ok(());
    };

    let t1_hc : Vec<HcPlayer> = t1_idx.iter().map(|idx| {
        ps_hc[*idx]
    }).collect();
    let t2_hc : Vec<HcPlayer> = t2_idx.iter().map(|idx| {
        ps_hc[*idx]
    }).collect();

    let (bat_team, bowl_team) = 
        if (is1win && iswinbat)
        || (!is1win && !iswinbat) {
            (t1_hc, t2_hc)
        } else {
            (t1_hc, t2_hc)
        };

    let result = self::start_match(
        &ctx,
        channel_id,
        bat_team,
        bowl_team,
        options,
        false
    ).await;

    fix_status(
        &ctx,
        &ps_id[..],
        false
    ).await?;

    result?;
    return Ok(());

    async fn pick<'a>(
        ctx: &&Context,
        channel_id: &ChannelId,
        cap: &User,
        ps_hc: &Vec<HcPlayer<'a>>,
        ps_idx: &mut Vec<usize>,
        t_idx: &mut Vec<usize>
     ) -> Result<(), String> {
        if ps_idx.len() == 1 {
            t_idx.push(
                ps_idx.remove(0)
            );
            return Ok(());
        }

        let mut avail = String::new();

        for idx in ps_idx.iter() {
            let name = match ps_hc[*idx] {
                HcPlayer::U(u) => u.name.clone(),
                HcPlayer::E(_) => String::from("Extrawicket")
            };
            avail.push_str(format!(
                "{},\n",
                name
            ).as_str());
        }

        let _ = channel_id.send_message(&ctx, |m| {
            m.content(format!(
                "Choose your team member by pinging: <@{}>, available players are:\n{}",
                cap.id,
                &avail
            ))
        }).await
        .map_err(|e| e.to_string())
        ?;
        
        let mut stream = channel_id
            .await_replies(&ctx)
            .author_id(cap.id)
            .timeout(Duration::new(TIMEOUT, 0))
            .build();

        while let Some(rep) = stream.next().await {
            if rep.mentions.len() == 0 {
                continue;
            }

            let c_idx = ps_idx
                .iter()
                .position(|idx| {
                    match ps_hc[*idx] {
                        HcPlayer::U(u) => 
                            rep.mentions.len() > 0
                            && u.id == rep.mentions[0].id,
                        HcPlayer::E(_) => rep.content.contains("extra")
                    }
                });

            if let Some(idx) = c_idx {
                ps_idx.remove(idx);
                t_idx.push(idx);
                break;
            } else {
                rep.reply(&ctx, format!(
                    "That is not a player available for pick. Try again from\n {}",
                    &avail
                )).await
                .map_err(|e| e.to_string())
                ?;
                continue;
            }
        }
        stream.stop();

        Ok(())
    }
}