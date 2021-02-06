use mongodb::bson::doc;
use rowifi_framework::prelude::*;

use super::ToggleOption;

#[derive(FromArgs)]
pub struct UpdateOnJoinArguments {
    #[arg(help = "Option to toggle the `Update on Join` setting")]
    pub option: ToggleOption,
}

pub async fn update_on_join(ctx: CommandContext, args: UpdateOnJoinArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let option = args.option;
    let (option, desc) = match option {
        ToggleOption::Enable => (true, "Update on Join has succesfully been enabled"),
        ToggleOption::Disable => (false, "Update on Join has successfully been disabled"),
    };

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$set": {"Settings.UpdateOnJoin": option}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Settings Modification Successful")
        .unwrap()
        .description(desc)
        .unwrap()
        .build()
        .unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .unwrap()
        .description(format!(
            "Settings Modification: Update On Join - {} -> {}",
            guild.settings.update_on_join, option
        ))
        .unwrap()
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}

#[derive(FromArgs)]
pub struct UpdateOnVerifyArguments {
    #[arg(help = "Option to toggle the `Update on Join` setting")]
    pub option: ToggleOption,
}

pub async fn update_on_verify(ctx: CommandContext, args: UpdateOnVerifyArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let (option, desc) = match args.option {
        ToggleOption::Enable => (true, "Update on Verify has succesfully been enabled"),
        ToggleOption::Disable => (false, "Update on Verify has successfully been disabled"),
    };

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$set": {"Settings.UpdateOnVerify": option}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Settings Modification Successful")
        .unwrap()
        .description(desc)
        .unwrap()
        .build()
        .unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .unwrap()
        .description(format!(
            "Settings Modification: Update On Verify - {} -> {}",
            guild.settings.update_on_verify, option
        ))
        .unwrap()
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
