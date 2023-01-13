use serenity::client::Context;
use serenity::framework::standard::{
    Args,
    macros::*
};
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::Message;

use crate::constants::{
    BANNER_URL,
    EMBED_COLOR,
    MAIN_SERVER,
    BOT_INVITE
};
use crate::models::Commands;

#[command]
///`e.help [command]`
///Let you know about all commands.
///None
#[bucket = "primary"]
pub async fn help(
    ctx: &Context,
    msg: &Message,
    mut args: Args
) -> CommandResult {
    let general_cmds = Vec::from(
        ["ping", "help"]
    );
    let cricket_cmds = Vec::from(
        ["rank", "hc"]
    );
    
    let user_id = msg.author.id;
    let avatar = msg.author.avatar.clone().unwrap_or(String::from(""));
    let avatar = format!("https://cdn.discordapp.com/avatars/{}/{}.webp", user_id, avatar);

    let cmds = {
        let data = ctx.data.read().await;
        data.get::<Commands>()
            .ok_or("Failed to get commands from context")
            ?.clone()
    };

    if let Ok(cmd) = args.single::<String>() {
        let cmd_found = cmds.get(&cmd)
            .or_else(|| {
                cmds.values().find(
                    |x| x[2].contains(&cmd)
                )
            });

        if let Some(value) = cmd_found {
            msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(
                        format!("Command - {}", cmd)
                    )
                    .color(EMBED_COLOR)
                    .description(
                        &value[1]
                    ).field(
                        "Syntax:",
                        &value[0],
                        false
                    ).field(
                        "Aliases",
                        &value[2],
                        false
                    ).footer(|f| {
                        f.text(format!(
                            "Requested by {}",
                            msg.author.name
                        )).icon_url(avatar)
                    })
                })
            }).await?;
        } else {
            msg.reply(&ctx.http, format!(
                "Command with name `{}` not found. Check for typos or checkout full help - `e.help`.",
                cmd
            )).await?;
        }
        return Ok(());
    }

    let get_txt = | cs : Vec<&str> | -> String {
        let mut txt = String::new();

        cs.into_iter().for_each(|c| {
            let info = cmds.get(c).unwrap();
            txt.push_str(format!(
                "**{}**  |  {}\n",
                c, info[0]
            ).as_str());
        });
        txt
    };
    
    let _ = msg.channel_id.send_message(&ctx, |m| {
        m.content(format!(
            "Here we have the help <@{}>.",
            user_id
        )).embed(|e| {
            e.title("Dispo Help")
            .description("Dispo is a handcricket bot that you can get your hands on to kill time. \n **_Prefix is `e.` ever_**")
            .field(
                "General Commands:",
                get_txt(general_cmds),
                false
            ).field(
                "Cricket Commands:",
                get_txt(cricket_cmds),
                false
            ).field("Links", format!(
                "[Add to Server]({})\n[Main Server]({})",
                BOT_INVITE, MAIN_SERVER
            ), false)
            .color(EMBED_COLOR)
            .image(BANNER_URL)
            .footer(|f| {
                f.text(format!(
                    "Requested by {}",
                    msg.author.name
                )).icon_url(avatar)
            })
        })
    }).await?;

    Ok(())
}
