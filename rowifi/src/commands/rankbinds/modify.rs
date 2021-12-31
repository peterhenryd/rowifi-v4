use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{BindType, Rankbind},
    id::RoleId,
    roblox::id::GroupId,
};

use super::new::PREFIX_REGEX;

#[derive(FromArgs)]
pub struct ModifyRankbind {
    #[arg(
        help = "The field to modify. Must be one of `priority` `roles-add` `roles-remove` `template`"
    )]
    pub option: ModifyOption,
    #[arg(help = "The Group ID of the rankbind to modify")]
    pub group_id: i64,
    #[arg(help = "The Rank ID of the rankbind to modify")]
    pub rank_id: i64,
    #[arg(help = "The actual modification to be made", rest)]
    pub change: String,
}

pub enum ModifyOption {
    Priority,
    RolesAdd,
    RolesRemove,
    Template,
}

pub async fn rankbinds_modify(ctx: CommandContext, args: ModifyRankbind) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let rankbinds = ctx
        .bot
        .database
        .query::<Rankbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type = $2",
            &[&(guild_id), &BindType::Rank],
        )
        .await?;

    let group_id = args.group_id;
    let rank_id = args.rank_id;

    let bind = match rankbinds
        .iter()
        .find(|r| r.group_id == group_id && r.group_rank_id == rank_id)
    {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Rank Bind Modification Failed")
                .description(format!(
                    "There was no bind found with Group Id {} and Rank Id {}",
                    group_id, rank_id
                ))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let name = format!("Group Id: {}", bind.group_id);
    let desc = match args.option {
        ModifyOption::Priority => {
            let priority = match args.change.parse::<i32>() {
                Ok(p) => p,
                Err(_) => {
                    let embed = EmbedBuilder::new()
                        .default_data()
                        .color(Color::Red as u32)
                        .title("Rank Bind Modification Failed")
                        .description("Priority was not found to be a number")
                        .build()
                        .unwrap();
                    ctx.respond().embeds(&[embed])?.exec().await?;
                    return Ok(());
                }
            };
            let new_priority = modify_priority(&ctx, bind, priority).await?;
            format!("`Priority`: {} -> {}", bind.priority, new_priority)
        }
        ModifyOption::RolesAdd => {
            let role_ids = add_roles(&ctx, bind, &args.change).await?;
            let modification = role_ids
                .iter()
                .map(|r| format!("<@&{}> ", r))
                .collect::<String>();
            format!("Added Roles: {}", modification)
        }
        ModifyOption::RolesRemove => {
            let role_ids = remove_roles(&ctx, bind, &args.change).await?;
            let modification = role_ids
                .iter()
                .map(|r| format!("<@&{}> ", r))
                .collect::<String>();
            format!("Removed Roles: {}", modification)
        }
        ModifyOption::Template => {
            if args.change.is_empty() {
                let embed = EmbedBuilder::new()
                    .default_data()
                    .color(Color::Red as u32)
                    .title("Rank Bind Modification Failed")
                    .description("You have entered a blank template")
                    .build()
                    .unwrap();
                ctx.respond().embeds(&[embed])?.exec().await?;
                return Ok(());
            }
            let template = modify_template(&ctx, group_id, rank_id, bind, &args.change).await?;
            format!("`New Template`: {}", template)
        }
    };
    let desc = format!("Rank Id: {}\n{}", bind.group_rank_id, desc);

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Success!")
        .description("The bind was successfully modified")
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Rank Bind Modification")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}

async fn modify_priority(
    ctx: &CommandContext,
    bind: &Rankbind,
    priority: i32,
) -> Result<i32, RoError> {
    ctx.bot
        .database
        .execute(
            "UPDATE binds SET priority = $1 WHERE bind_id = $2",
            &[&priority, &bind.bind_id],
        )
        .await?;
    Ok(priority)
}

async fn modify_template<'t>(
    ctx: &CommandContext,
    group_id: i64,
    rank_id: i64,
    bind: &Rankbind,
    template: &'t str,
) -> Result<String, RoError> {
    let roblox_group = ctx
        .bot
        .roblox
        .get_group_ranks(GroupId(group_id as u64))
        .await?;
    let roblox_rank = match &roblox_group {
        Some(g) => g.roles.iter().find(|r| i64::from(r.rank) == rank_id),
        None => None,
    };
    let template = match template {
        "auto" => {
            if let Some(rank) = roblox_rank {
                if let Some(m) = PREFIX_REGEX.captures(&rank.name) {
                    format!("[{}] {{roblox-username}}", m.get(1).unwrap().as_str())
                } else {
                    "{roblox-username}".into()
                }
            } else {
                "{roblox-username}".into()
            }
        }
        "disable" => "{discord-name}".into(),
        "N/A" => "{roblox-username}".into(),
        _ => template.to_string(),
    };
    ctx.bot
        .database
        .execute(
            "UPDATE binds SET template = $1 WHERE bind_id = $2",
            &[&template, &bind.bind_id],
        )
        .await?;
    Ok(template)
}

async fn add_roles(
    ctx: &CommandContext,
    bind: &Rankbind,
    roles: &str,
) -> Result<Vec<RoleId>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();
    ctx.bot.database.execute("UPDATE binds SET discord_roles = array_cat(discord_roles, $1::BIGINT[]) WHERE bind_id = $2", &[&role_ids, &bind.bind_id]).await?;
    Ok(role_ids)
}

async fn remove_roles(
    ctx: &CommandContext,
    bind: &Rankbind,
    roles: &str,
) -> Result<Vec<RoleId>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();
    let mut roles_to_keep = bind.discord_roles.clone();
    roles_to_keep.retain(|r| !role_ids.contains(r));
    ctx.bot
        .database
        .execute(
            "UPDATE binds SET discord_roles = $1 WHERE bind_id = $2",
            &[&roles_to_keep, &bind.bind_id],
        )
        .await?;
    Ok(role_ids)
}

impl FromArg for ModifyOption {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        match arg.to_ascii_lowercase().as_str() {
            "priority" => Ok(ModifyOption::Priority),
            "roles-add" => Ok(ModifyOption::RolesAdd),
            "roles-remove" => Ok(ModifyOption::RolesRemove),
            "template" => Ok(ModifyOption::Template),
            _ => Err(ParseError(
                "one of `priority` `roles-add` `roles-remove` `template`",
            )),
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match &option.value {
            CommandOptionValue::String(value) => value.to_string(),
            CommandOptionValue::Integer(value) => value.to_string(),
            _ => unreachable!("ModifyArgumentRankbinds unreached"),
        };

        ModifyOption::from_arg(&arg)
    }
}
