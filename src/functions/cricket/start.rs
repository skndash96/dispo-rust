use serenity::{
    model::id::ChannelId,
    builder::CreateEmbed
};
use serenity::client::Context;

use crate::constants::EMBED_COLOR;
use crate::models::{
    HcOptions,
    HcPlayer
};

pub async fn start_match<'a> (
    ctx: &Context,
    channel_id: ChannelId,
    bat_team: Vec<HcPlayer<'a>>,
    bowl_team: Vec<HcPlayer<'a>>,
    options: HcOptions,
    is_duo: bool
) -> Result<(), String> {
    let hc_typ = if is_duo { "duo" } else { "team" };

    let overs = options.overs;
    let wickets = options.wickets;
    let post = options.post;

    let mut bat : HcPlayer = bat_team[0]; 
    let mut bowl : HcPlayer = bowl_team[0];
    let mut ball_no : u16 = 0;
    let mut bat_scr : u16 = 0;
    let mut team_scr : u16 = 0;
    let mut is_in1 : bool = true;
    let mut in1_bat : Vec<u16> = vec![];
    let mut in2_bat : Vec<u16> = vec![];
    let mut ducks : Vec<String> = vec![];

    let send = || async {
        let txt = String::new();

        // let mut idx = 0;
        // for p in bat_team {
        //     let name = match p {
        //         HcPlayer::U(u) => u.name,
        //         HcPlayer::E(u) => String::from("extra")
        //     };
        //     let scr = if is_in1 { in1_bat[idx] };

        //     idx += 1;
        // }

        let mut embed = CreateEmbed::default();
        embed
            .title(format!(
                "{} Match",
                hc_typ
            ))
            .description("Let the match begin!")
            .color(EMBED_COLOR);//TODO

        let bat_u = match bat {
            HcPlayer::E(u) => u.0.unwrap(),
            HcPlayer::U(u) => u
        };
        let bowl_u = match bowl {
            HcPlayer::E(u) => u.0.unwrap(),
            HcPlayer::U(u) => u
        };

        if post {
            channel_id.send_message(&ctx, |m| m.set_embed(embed.clone()))
            .await
            .map_err(|e| e.to_string())
            ?;
        }

        bat_u.dm(&ctx, |m| m.set_embed(embed.clone()))
            .await
            .map_err(|e| e.to_string())
            ?;
        bowl_u.dm(&ctx, |m| m.set_embed(embed.clone()))
            .await
            .map_err(|e| e.to_string())
            ?;

        Ok::<(), String>(())
    };

    Ok(())
}
