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

    let mut ps_id : Vec<String> = vec![];

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
                .map_err(|e| e.to_string())
                ?;

            msg.reply(&ctx, format!(
                "Team Match cancelled by **{}**.",
                from.name
            )).await
            .map_err(|e| e.to_string())
            ?;

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

                msg.channel_id.say(&ctx, format!(
                    "**{}** is already engaged in a match.",
                    from.name
                )).await
                .map_err(|e| e.to_string())
                ?;
            };
        } else if let Some(idx) = 
            ps_id.iter().position(
                |id| *id == from.id.to_string()
            ) {
            if let Err(why) = fix_status(
                &ctx,
                &ps_id[idx..idx+1],
                false
            ).await {
                eprintln!("Failed to fix status to false in hc team collect: {:?}", why);
            }

            ps_id.remove(idx);
        }

        msg.channel_id.say(&ctx, format!(
            "{} **{}** - {} the team match.",
            if is_added {"☑️"} else {"❎"},
            from.name,
            if is_added { "joined" } else { "left"}
        )).await
        .map_err(|e| e.to_string())
        ?;
    }

    stream.stop();

    let start_match = || async {
        let mut ps = talk_msg.reaction_users(
            &ctx,
            enter_em,
            None,
            None
        ).await
        .map_err(|e| e.to_string())
        ?;
        ps.drain_filter(|p| p.bot/*removes all bots*/);
        let ps = ps;

        if ps.len() > 22 {
            talk_msg.reply(&ctx,
                "A maximum of 22 players can play a team match at once. The count is too high! Try with a fewer members."
            ).await
            .map_err(|e| e.to_string())
            ?;

            return Ok(());
        } else if ps.len() < 3 {
            talk_msg.reply(&ctx,
                "Aw poor, a minimum of 3 players are required to start a team match."
            ).await
            .map_err(|e| e.to_string())
            ?;

            return Ok(());
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
                "Teaming: We got,\n Captain 1: **{}**\n Captain 2: **{}**.",
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
    
        let last_p_idx = ps_hc.len()-1;
        if let HcPlayer::E(_) = ps_hc[last_p_idx] {
            let et1 = t1_idx.contains(&last_p_idx);
            let cap = if et1 {&cap1} else {&cap2};
            let t_idx = if et1 {&t1_idx} else {&t2_idx};
            let avail = get_avail_txt(
                &ps_hc,
                &t_idx
            );

            channel_id.say(&ctx, format!(
                "<@{}>, Who is going to play Extrawicket in your team? Ping them. Team members are:\n{}",
                cap.id,
                &avail
            ))
            .await
            .map_err(|e| e.to_string())
            ?;

            loop {
                let rep = channel_id
                    .await_reply(&ctx)
                    .timeout(Duration::new(TIMEOUT, 0))
                    .author_id(cap.id)
                    .await;
    
                if let Some(val) = rep {
                    if val.mentions.len() == 0 {
                        continue;
                    }
                    let c = &val.mentions[0];
    
                    let idx = ps/*not ps_hc, coz either way same idx*/
                        .iter().position(|p| {
                            p.id == c.id
                        });
    
                    if idx.is_some()
                    && t_idx.contains(&idx.unwrap())
                       {
                        ps_hc[last_p_idx] = HcPlayer::E(
                            Extra(Some(&ps[idx.unwrap()]))
                        );
                        break;
                    } else {
                        val.reply(&ctx, format!(
                            "Mentioned player is not valid player in your team. Team members are:\n{}",
                            &avail
                        ))
                        .await
                        .map_err(|e| e.to_string())
                        ?;
                        continue;
                    }
                } else {
                    return Err(format!(
                        "res: {}",
                        cap.name
                    ));
                }
            }
        }

        //TODO: Get ordering of teams.

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
            .timeout(Duration::new(TIMEOUT, 0))
            .await;
    
        let iswinbat = if let Some(res) = toss_res {
            match res.data.custom_id.as_str() {
                "bat" => true,
                _/*"bowl"*/ => false
            }
        } else {
            toss_msg.delete(&ctx)
                .await
                .map_err(|e| e.to_string())
                ?;
    
            return Err(format!(
                "res: {}",
                if is1win {cap1.name.clone()} else {cap2.name.clone()}
            ));
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
    
        self::start_match(
            &ctx,
            channel_id,
            bat_team,
            bowl_team,
            options,
            false
        ).await?;

        return Ok(());
    };

    let start_res = start_match().await;
    fix_status(&ctx, &ps_id[..], false)
        .await?;
    start_res?;

    return Ok(());

    fn get_avail_txt (
        ps_hc: &Vec<HcPlayer>,
        ps_idx: &Vec<usize>
    ) -> String {
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

        return avail;
    }

    async fn pick<'a>(
        ctx: &&Context,
        channel_id: &ChannelId,
        cap: &User,
        ps_hc: &Vec<HcPlayer<'a>>,
        ps_idx: &mut Vec<usize>,
        t_idx: &mut Vec<usize>
     ) -> Result<(), String> {
        println!("{:?}", ps_idx);

        if ps_idx.len() == 1 {
            t_idx.push(
                ps_idx.remove(0)
            );
            return Ok(());
        }

        let avail = get_avail_txt(
            &ps_hc,
            &ps_idx
        );

        let _ = channel_id.send_message(&ctx, |m| {
            m.content(format!(
                "Choose your team member by pinging: <@{}>, available players are:\n{}",
                cap.id,
                &avail
            ))
        }).await
        .map_err(|e| e.to_string())
        ?;

        loop {
            let rep = channel_id
                .await_reply(&ctx)
                .author_id(cap.id)
                .timeout(Duration::new(TIMEOUT, 0))
                .await;

            if rep.is_none() {
                return Err(format!(
                    "res: {}",
                    cap.name
                ));
            }

            let rep = rep.unwrap();

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

        Ok(())
    }
}