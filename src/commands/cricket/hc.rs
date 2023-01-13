use serenity::model::channel::Message;
use serenity::client::Context;
use serenity::framework::standard::macros::*;
use serenity::framework::standard::{
    Args,
    CommandResult
};

use crate::utils::{
    get_player_data,
    check_status,
    fix_status
};
use crate::models::{
    Players,
    HcOptions,
    Engaged
};
use crate::functions::cricket::{
    duo,
    team,
    get_hc_options
};

#[command]
///`e.hc <@user | team>`
///Play handcricket with friend(s)
///cricket
#[min_args(1)]
#[aliases(cricket)]
pub async fn hc(
    ctx: &Context,
    msg: &Message,
    mut args: Args
) -> CommandResult {
    let db = {
        let data = ctx.data.read().await;
        data.get::<Players>()
            .ok_or(String::from(
                "Failed to get players data from context."
            ))?
            .clone()
    };

    let sub_cmd = args.single::<String>()
        .map_err(|e| e.to_string())
        ?;
    let options : HcOptions = match get_hc_options(&mut args) {
        Ok(val) => val,
        Err(why) => {
            msg.reply(&ctx, why)
                .await
                .map_err(|e| e.to_string())
                ?;
            return Ok(())
        }
    };

    let u = &msg.author;
    let u_data = get_player_data(&db, u.id.to_string()).await?;

    if sub_cmd == "team" {
        //TODO TEAM
        return Ok(());
    } else {
        //DUO
        if msg.mentions.len() == 0 {
            Err("syntax")?;
        }

        let t = &msg.mentions[0];
        if t.id == u.id {
            Err("self_mention")?;
        }
        let t_data = get_player_data(&db, t.id.to_string()).await?;

        let ps_id = [u, t].iter().map(|u| u.id.to_string()).collect();

        {
            let mut data = ctx.data.write().await;
            let engaged = data.get_mut::<Engaged>()
                .ok_or("Failed to get engaged users data from context")
                ?;

            if let Err(why) = check_status(
                &engaged,
                &ps_id
            ) {
                Err(why)
            } else {
                (*engaged).extend_from_slice(&ps_id);
                Ok(())
            }
        }?;

        let result = duo::set_match(
            &ctx,
            &msg,
            &u,
            &t,
            options
        ).await;

        fix_status(&ctx, &ps_id)
            .await?;

        result?;
        Ok(())
    }
}
