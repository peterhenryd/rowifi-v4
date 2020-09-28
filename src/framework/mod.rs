pub mod context;
mod map;
pub mod parser;
pub mod prelude;
pub mod structures;

use dashmap::DashMap;
use std::time::Duration;
use transient_dashmap::TransientDashMap;
use twilight_gateway::Event;
use twilight_model::{
    channel::Message,
    guild::Permissions
};
use twilight_command_parser::Arguments;
use uwl::Stream;
use crate::utils::{
    error::{RoError, CommandError}, 
    misc::guild_wide_permissions
};

use context::Context;
pub use map::CommandMap;
use structures::*;
use parser::{ParseError, Invoke};

#[derive(Default)]
pub struct Framework {
    commands: Vec<(&'static Command, CommandMap)>,
    buckets: DashMap<String, Bucket>,
    help: Option<&'static HelpCommand>,
}

impl Framework {
    pub fn command(mut self, command: &'static Command) -> Self {
        let map = CommandMap::new(&[command]);
        self.commands.push((command, map));
        self
    }

    pub fn help(mut self, help: &'static HelpCommand) -> Self {
        self.help = Some(help);
        self
    }

    pub fn bucket(self, name: &str, time: Duration, calls: u64) -> Self {
        self.buckets.insert(name.to_string(), Bucket {time, guilds: TransientDashMap::new(time), calls});
        self
    }

    async fn dispatch(&self, msg: Message, mut context: Context) {
        if msg.author.bot || msg.webhook_id.is_some() || msg.guild_id.is_none() || msg.content.is_empty() {
            return;
        }

        let mut stream = Stream::new(&msg.content);
        stream.take_while_char(|c| c.is_whitespace());

        let prefix = parser::find_prefix(&mut stream, &msg, context.config.as_ref()).await;
        if prefix.is_some() && stream.rest().is_empty() {
            let command_prefix = context.config.prefixes.get(&msg.guild_id.unwrap()).map(|g| g.to_string()).unwrap_or_else(|| context.config.default_prefix.to_string());
            let _ = context.http.create_message(msg.channel_id).content(format!("The prefix of this server is {}", command_prefix)).unwrap().await;
            return;
        }

        if prefix.is_none() {
            return;
        }

        let invocation = parser::command(&mut stream, &self.commands, &self.help.as_ref().map(|h| h.name)).await;
        let invoke = match invocation {
            Ok(i) => i,
            Err(ParseError::UnrecognisedCommand(_)) => {
                return;
            }
        };

        match invoke {
            Invoke::Help => {
                let args = Arguments::new(stream.rest());
                if let Some(help) = self.help {
                    let _res = (help.fun)(&mut context, &msg, args, &self.commands).await;
                }
            },
            Invoke::Command{command} => {
                if !self.run_checks(&context, &msg, command) {
                    return;
                }

                if let Some(bucket) = command.options.bucket.and_then(|b| self.buckets.get(b)) {
                    if let Some(duration) = bucket.get(msg.guild_id.unwrap()) {
                        let content = format!("Ratelimit reached. You may use this command in {:?}", duration);
                        let _ = context.http.create_message(msg.channel_id).content(content).unwrap().await;
                        return;
                    }
                }

                let args = Arguments::new(stream.rest());
                let args_count = Arguments::new(stream.rest()).count();
                if args_count < command.options.min_args {
                    let content = format!("```{}\n\n Expected atleast {} arguments, got only {}```", msg.content, command.options.min_args, args_count);
                    let _ = context.http.create_message(msg.channel_id).content(content).unwrap().await;
                    return;
                }

                let res = (command.fun)(&mut context, &msg, args).await;

                match res {
                    Ok(()) => {
                        if let Some(bucket) = command.options.bucket.and_then(|b| self.buckets.get(b)) {
                            bucket.take(msg.guild_id.unwrap()); 
                        }
                    },
                    Err(error) => self.handle_error(error, &context, &msg).await
                }
            }
        }
    }

    pub async fn handle_event(&self, event: Event, context: Context) {
        if let Event::MessageCreate(msg) = event {
            self.dispatch(msg.0, context).await;
        }
    }

    fn run_checks(&self, context: &Context, msg: &Message, command: &Command) -> bool {
        if context.config.blocked_users.contains(&msg.author.id) {
            return false;
        }

        if context.config.blocked_guilds.contains(&msg.guild_id.unwrap()) {
            return false;
        }

        if context.config.owners.contains(&msg.author.id) {
            return true;
        }

        if context.config.disabled_channels.contains(&msg.channel_id) && !command.options.names.contains(&"command-channel") {
            return false;
        }

        if let Some(guild) = context.cache.guild(msg.guild_id.unwrap()) {
            if context.config.blocked_users.contains(&guild.owner_id) {
                return false;
            }

            if msg.author.id.0 == guild.owner_id.0 {
                return true;
            }

            if let Some(member) = context.cache.member(guild.id, msg.author.id) {
                match command.options.perm_level {
                    RoLevel::Normal => return true,
                    RoLevel::Creator => {
                        if context.config.owners.contains(&msg.author.id) {
                            return true;
                        }
                    },
                    RoLevel::Admin => {
                        match guild_wide_permissions(&context, msg.guild_id.unwrap(), member.user.id, &member.roles) {
                            Ok(permissions) => {
                                if permissions.contains(Permissions::MANAGE_GUILD) {
                                    return true;
                                }
                            },
                            Err(why) => tracing::error!(guild = ?msg.guild_id, reason = ?why)
                        };
                        if let Some(admin_role) = guild.admin_role {
                            if member.roles.contains(&admin_role) {
                                return true;
                            }
                        }
                    },
                    RoLevel::Trainer => return true
                }
            }
        }
        false
    }

    async fn handle_error(&self, error: RoError, context: &Context, msg: &Message) {
        match error {
            RoError::Command(cmd_err) => {
                match cmd_err {
                    CommandError::Blacklist(reason) => {
                        let _ = context.http.create_message(msg.channel_id)
                            .content(format!("User was found on the server blacklist. Reason: {}", reason)).unwrap()
                            .await;
                    },
                    CommandError::NicknameTooLong(nick) => {
                        let _ = context.http.create_message(msg.channel_id)
                            .content(format!("The supposed nickname {} was found to be longer than 32 characters", nick)).unwrap()
                            .await;
                    },
                    CommandError::NoRoGuild => {
                        let _ = context.http.create_message(msg.channel_id)
                            .content("This server was not set up. Please ask the server owner to run `setup`").unwrap()
                            .await;
                    }
                    CommandError::ParseArgument(arg, param, param_type) => {
                        let idx = msg.content.find(&arg).unwrap();
                        let size = arg.len();
                        let content = format!("```{}\n{}{}\n\nExpected {} to be a {}```", 
                            msg.content, " ".repeat(idx), "^".repeat(size), param, param_type
                        );
                        let _ = context.http.create_message(msg.channel_id)
                            .content(content).unwrap().await;
                    },
                    CommandError::Timeout => {
                        let _ = context.http.create_message(msg.channel_id)
                            .content("Timeout reached. Please try again").unwrap().await;
                    }
                }
            },
            _ => {
                let _ = context.http.create_message(msg.channel_id)
                    .content("There was an error in executing this command. Please try again. If the issue persists, please contact the support server for more information").unwrap()
                    .await;
            }
        }
    }
}