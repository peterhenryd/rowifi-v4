use itertools::Itertools;
use rowifi_database::postgres::Row;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{AssetType, Assetbind, BindType, Template},
    discord::id::RoleId,
};

#[derive(FromArgs)]
pub struct NewArguments {
    #[arg(help = "The type of asset to create")]
    pub option: AssetType,
    #[arg(help = "The ID of asset to bind")]
    pub asset_id: i64,
    #[arg(help = "The template to be used for the bind. Can be initialized as `N/A`, `disable`")]
    pub template: String,
    #[arg(help = "The number that tells the bot which bind to choose for the nickname")]
    pub priority: Option<i32>,
    #[arg(help = "The Discord Roles to add to the bind", rest)]
    pub discord_roles: Option<String>,
}

pub async fn assetbinds_new(ctx: CommandContext, args: NewArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let assetbinds = ctx
        .bot
        .database
        .query::<Assetbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY asset_id",
            &[&(guild_id.get() as i64), &BindType::Asset],
        )
        .await?;

    let asset_type = args.option;
    let asset_id = args.asset_id;
    if assetbinds
        .iter()
        .any(|a| a.asset_type == asset_type && a.asset_id == asset_id)
    {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Bind Addition Failed")
            .color(Color::Red as u32)
            .description(format!("A bind with asset id {} already exists", asset_id))
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let template = args.template;
    let template_str = match template.as_str() {
        "disable" => "{discord-name}".into(),
        "N/A" => "{roblox-username}".into(),
        _ => {
            if Template::has_slug(template.as_str()) {
                template.clone()
            } else {
                format!("{} {{roblox-username}}", template)
            }
        }
    };

    let priority = args.priority.unwrap_or_default();

    let discord_roles_str = args.discord_roles.unwrap_or_default();
    let roles_to_add = discord_roles_str
        .split_ascii_whitespace()
        .collect::<Vec<_>>();

    let server_roles = ctx.bot.cache.roles(guild_id);
    let mut roles = Vec::new();
    for r in roles_to_add {
        if let Some(resolved) = &ctx.resolved {
            roles.extend(resolved.roles.iter().map(|r| r.0.get() as i64));
        } else if let Some(role_id) = parse_role(r) {
            if server_roles.contains(&RoleId::new(role_id).unwrap()) {
                roles.push(role_id as i64);
            }
        }
    }

    let bind = Assetbind {
        // 0 is entered here since this field is not used in the insertion. The struct is only constructed to ensure we have
        // collected all fields.
        bind_id: 0,
        asset_id,
        asset_type,
        discord_roles: roles.into_iter().unique().collect::<Vec<_>>(),
        priority,
        template: Template(template_str.clone()),
    };

    let row = ctx.bot.database.query_one::<Row>(
        "INSERT INTO binds(bind_type, guild_id, asset_id, asset_type, discord_roles, priority, template) VALUES($1, $2, $3, $4, $5, $6, $7) RETURNING bind_id",
        &[&BindType::Asset, &(guild_id.get() as i64), &bind.asset_id, &bind.asset_type, &bind.discord_roles, &bind.priority, &bind.template]
    ).await?;
    let bind_id: i64 = row.get("bind_id");

    let name = format!("Id: {}", asset_id);
    let value = format!(
        "Type: {}\nTemplate: `{}`\nPriority: {}\nRoles: {}",
        bind.asset_type,
        template_str,
        priority,
        bind.discord_roles
            .iter()
            .map(|r| format!("<@&{}> ", r))
            .collect::<String>()
    );
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Bind Addition Successful")
        .color(Color::DarkGreen as u32)
        .field(EmbedFieldBuilder::new(name.clone(), value.clone()))
        .build()
        .unwrap();
    let message = ctx
        .respond()
        .embeds(&[embed])?
        .components(&[Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                style: ButtonStyle::Danger,
                emoji: Some(ReactionType::Unicode {
                    name: "🗑️".into()
                }),
                label: Some("Oh no! Delete?".into()),
                custom_id: Some("ab-new-delete".into()),
                url: None,
                disabled: false,
            })],
        })])?
        .exec()
        .await?
        .model()
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Asset Bind Addition")
        .field(EmbedFieldBuilder::new(name, value))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    let message_id = message.id;
    let author_id = ctx.author.id;

    let stream = ctx
        .bot
        .standby
        .wait_for_component_interaction(message_id)
        .timeout(Duration::from_secs(60));
    tokio::pin!(stream);

    ctx.bot.ignore_message_components.insert(message_id);
    while let Some(Ok(event)) = stream.next().await {
        if let Event::InteractionCreate(interaction) = &event {
            if let Interaction::MessageComponent(message_component) = &interaction.0 {
                let component_interaction_author = message_component.author_id().unwrap();
                if component_interaction_author == author_id {
                    ctx.bot
                        .http
                        .interaction_callback(
                            message_component.id,
                            &message_component.token,
                            &InteractionResponse::UpdateMessage(CallbackData {
                                allowed_mentions: None,
                                content: None,
                                components: Some(Vec::new()),
                                embeds: Vec::new(),
                                flags: None,
                                tts: None,
                            }),
                        )
                        .exec()
                        .await?;

                    ctx.bot
                        .database
                        .execute("DELETE FROM binds WHERE bind_id = $1", &[&bind_id])
                        .await?;

                    let embed = EmbedBuilder::new()
                        .default_data()
                        .color(Color::DarkGreen as u32)
                        .title("Successful!")
                        .description("The newly created bind was deleted")
                        .build()
                        .unwrap();
                    ctx.bot
                        .http
                        .create_followup_message(&message_component.token)
                        .unwrap()
                        .embeds(&[embed])
                        .exec()
                        .await?;

                    break;
                }
                let _ = ctx
                    .bot
                    .http
                    .interaction_callback(
                        message_component.id,
                        &message_component.token,
                        &InteractionResponse::DeferredUpdateMessage,
                    )
                    .exec()
                    .await;
                let _ = ctx
                    .bot
                    .http
                    .create_followup_message(&message_component.token)
                    .unwrap()
                    .ephemeral(true)
                    .content("This button is only interactable by the original command invoker")
                    .exec()
                    .await;
            }
        }
    }
    ctx.bot.ignore_message_components.remove(&message_id);

    Ok(())
}
