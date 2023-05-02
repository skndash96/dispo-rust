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

    type Logs = Vec<[u16; 2 /*[score,balls]*/]>;
    let mut isin1 = true;
    let mut in1_logs : Logs = vec![];
    let mut in2_logs : Logs = vec![];
    let mut c_bat_logs : &Logs = &in1_logs;
    let mut c_bowl_logs : &Logs = &in2_logs;

    let mut strike : u32 = 0;
    /*If above is even, accept ball. If above is odd, accept bat*/
    /*coz ball goes first and then bat*/

    let mut c_ball_no : u16 = 0;
    let mut c_bat_scr : u16 = 0;
    let mut c_team_scr : u16 = 0;
    let mut t_scr : Option<u16> = None;

    let mut c_bat_team : &Vec<HcPlayer> = &bat_team;
    let mut c_bowl_team : &Vec<HcPlayer> = &bowl_team;
    let mut c_bat_idx = 0_usize;
    let mut c_bowl_idx = 0_usize;

    let get_duo_txt = || {
        let mut txt = String::new();

        let bat_name = match c_bat_team[0] {
            HcPlayer::U(u) => u.name.clone(),
            HcPlayer::E(_) => {
                eprintln!("Duo match player is HcPlayer::E.");
                String::from("anonymous")
            }
        };
        let [bat_scr, bat_balls] = c_bat_logs[0];

        let bowl_name = match c_bat_team[0] {
            HcPlayer::U(u) => u.name.clone(),
            HcPlayer::E(_) => {
                eprintln!("Duo match player is HcPlayer::E.");
                String::from("anonymous")
            }
        };
        let [bowl_scr, bowl_balls] = c_bowl_logs[0];

        txt.push_str("**_Batsman_**");
        txt.push_str(format!(
            "\n{} | {} `({})`",
            bat_name,
            bat_scr,
            bat_balls
        ).as_str());

        if !isin1 {
            txt.push_str(format!(
                "\nTarget: {}",
                t_scr.unwrap()
            ).as_str());
        }

        txt.push_str("\n**_Bowler_**");
        txt.push_str(format!(
            "\n{} | {} `({})`",
            bowl_name,
            bowl_scr,
            bowl_balls
        ).as_str());

        txt
    };

    let pad = |a: &String| {
        if a.len() > 12 {
            format!("{}..", &a[..10]) /*take first 10 and add ..*/
        } else {
            format!("{:<12}", &a)
        }
    };

    let get_team_txt = || {
        let mut txt = String::new();

        txt.push_str("**__Batting Team__**");
        for (i, p) in c_bat_team.iter().enumerate() {
            let name = match p {
                HcPlayer::U(u) => u.name.clone(),
                HcPlayer::E(u) => format!("{} (extra)", u.0.unwrap().name.clone())
            };
            let name = pad(&name);
    
            let [scr, balls] = c_bat_logs[i];
            let isc = if i == c_bat_idx {"_"} else {""};

            txt.push_str(format!(
                "\n{}{}{} | {}`({})`",
                isc,
                name,
                isc,
                scr,
                balls
            ).as_str());
        }
        txt.push_str(format!(
            "\nTotal Score: {}",
            c_team_scr
        ).as_str());

        if !isin1 {
            txt.push_str(format!(
                "\nTarget: {}",
                t_scr.unwrap()
            ).as_str());
        }

        txt.push_str("\n\n**__Bowling Team__**");
        for (i, p) in c_bowl_team.iter().enumerate() {
            let name = match p {
                HcPlayer::U(u) => u.name.clone(),
                HcPlayer::E(u) => format!("{} (extra)", u.0.unwrap().name.clone())
            };
            let name = pad(&name);
            let [scr, balls] = c_bowl_logs[i];
            let isc = if i == c_bowl_idx {"_"} else {""};

            txt.push_str(format!(
                "\n{}{}{} | {}`({})`",
                isc,
                name,
                isc,
                scr,
                balls
            ).as_str());
        }

        txt
    };

    if is_duo {
        std::mem::drop(get_team_txt);
    } else {
        std::mem::drop(get_duo_txt);
    }

    let send = || async {
        let mut embed = CreateEmbed::default();
        embed
            .title(format!(
                "{} Match",
                hc_typ
            ))
            .description(
                if is_duo { get_duo_txt() } else { get_team_txt() }
            )
            .color(EMBED_COLOR);//TODO

        let bat_u = match c_bat_team[c_bat_idx] {
            HcPlayer::E(u) => u.0.unwrap(),
            HcPlayer::U(u) => u
        };
        let bowl_u = match c_bat_team[c_bat_idx] {
            HcPlayer::E(u) => u.0.unwrap(),
            HcPlayer::U(u) => u
        };

        if true /*posr*/ {
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

    //TODO start bat and ball threads
    send().await;

    Ok(())
}
