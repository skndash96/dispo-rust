use serenity::framework::standard::Args;

use crate::models::HcOptions;

pub mod duo;
pub mod team;
pub mod start;

pub fn get_hc_options(
    args: &mut Args
) -> Result<HcOptions, String> {
    let mut options = HcOptions {
        post: false,
        overs: 4,
        wickets: 1
    };

    while let Some(arg) = args.advance().current() {
        match arg {
            "--post" => {
                options.post = true;
            },
            "--wickets" => {
                let num = args.advance().parse::<u8>()
                    .map_err(|_| "Given value for `wickets` flag is not valid, try for a number from 1 to 10.")?;
                if num < 11 && num > 0 {
                    options.wickets = num;
                }
            },
            "--overs" => {
                let num = args.advance().parse::<u8>()
                    .map_err(|_| "Given value for `overs` flag is not valid, try for a number from 1 to 10.")?;
                if num < 11 && num > 0 {
                    options.overs = num;
                }
            },
            _other => {}
        };
    };
    
    Ok(options)
}
