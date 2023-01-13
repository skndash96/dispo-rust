use serenity::framework::standard::macros::*;

mod ping;
use ping::PING_COMMAND;

mod help;
use help::HELP_COMMAND;

#[group]
#[commands(ping, help)]
#[description = "All general commands like ping and help."]
#[help_available]
pub struct General;
