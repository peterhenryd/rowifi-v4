use itertools::Itertools;
use serde::{Serialize, Deserialize};
use serde_repr::*;
use std::{default::Default, fmt, collections::HashMap, sync::Arc};
use twilight_model::id::{RoleId, GuildId};

use super::{bind::*, blacklist::*};
use crate::cache::CachedRole;
use crate::framework::context::Context;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RoGuild {
    #[serde(rename = "_id")]
    pub id: i64,

    #[serde(rename = "Prefix")]
    pub command_prefix: Option<String>,

    #[serde(rename = "Settings")]
    pub settings: GuildSettings,

    #[serde(rename = "VerificationRole")]
    pub verification_role: i64,

    #[serde(rename = "VerifiedRole")]
    pub verified_role: i64,

    #[serde(rename = "RankBinds")]
    pub rankbinds: Vec<RankBind>,

    #[serde(rename = "GroupBinds")]
    pub groupbinds: Vec<GroupBind>,

    #[serde(rename = "CustomBinds")]
    #[serde(default)]
    pub custombinds: Vec<CustomBind>,

    #[serde(rename = "AssetBinds")]
    #[serde(default)]
    pub assetbinds: Vec<AssetBind>,

    #[serde(rename = "Blacklists")]
    #[serde(default)]
    pub blacklists: Vec<Blacklist>,

    #[serde(rename = "DisabledChannels")]
    #[serde(default)]
    pub disabled_channels: Vec<i64>
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BackupGuild {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,

    #[serde(rename = "UserId")]
    pub user_id: i64,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Prefix")]
    pub command_prefix: Option<String>,

    #[serde(rename = "Settings")]
    pub settings: GuildSettings,

    #[serde(rename = "VerificationRole")]
    pub verification_role: Option<String>,

    #[serde(rename = "VerifiedRole")]
    pub verified_role: Option<String>,

    #[serde(rename = "Rankbinds")]
    pub rankbinds: Vec<BackupRankBind>,

    #[serde(rename = "Groupbinds")]
    pub groupbinds: Vec<BackupGroupBind>,

    #[serde(rename = "Custombinds")]
    #[serde(default)]
    pub custombinds: Vec<BackupCustomBind>,

    #[serde(rename = "Assetbinds")]
    #[serde(default)]
    pub assetbinds: Vec<BackupAssetBind>,

    #[serde(rename = "Blacklists")]
    #[serde(default)]
    pub blacklists: Vec<Blacklist>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct GuildSettings {
    #[serde(rename = "AutoDetection")]
    pub auto_detection: bool,

    #[serde(rename = "Type")]
    pub guild_type: GuildType,

    #[serde(rename = "BlacklistAction")]
    #[serde(default)]
    pub blacklist_action: BlacklistActionType,

    #[serde(rename = "UpdateOnJoin")]
    #[serde(default)]
    pub update_on_join: bool,

    #[serde(rename = "UpdateOnVerify")]
    #[serde(default)]
    pub update_on_verify: bool
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Eq, PartialEq, Clone)]
#[repr(i8)]
pub enum GuildType {
    Alpha, Beta, Normal
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone)]
#[repr(i8)]
pub enum BlacklistActionType {
    None, Kick, Ban
}

impl Default for GuildType {
    fn default() -> Self {
        GuildType::Normal
    }
}

impl Default for BlacklistActionType {
    fn default() -> Self {
        BlacklistActionType::None
    }
}

impl fmt::Display for BlacklistActionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlacklistActionType::None => write!(f, "None"),
            BlacklistActionType::Kick=> write!(f, "Kick"),
            BlacklistActionType::Ban => write!(f, "Ban")
        }
    }
}

impl fmt::Display for GuildType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GuildType::Alpha => write!(f, "Alpha"),
            GuildType::Beta => write!(f, "Beta"),
            GuildType::Normal => write!(f, "Normal")
        }
    }
}

impl RoGuild {
    pub fn to_backup(&self, user_id: i64, name: &str, roles: &HashMap<RoleId, Arc<CachedRole>>) -> BackupGuild {
        let rankbinds = self.rankbinds.iter().map(|r| r.to_backup(roles)).collect_vec();
        let groupbinds = self.groupbinds.iter().map(|g| g.to_backup(roles)).collect_vec();
        let custombinds = self.custombinds.iter().map(|c| c.to_backup(roles)).collect_vec();
        let assetbinds = self.assetbinds.iter().map(|a| a.to_backup(roles)).collect_vec();

        BackupGuild {
            id: bson::oid::ObjectId::new(),
            user_id,
            name: name.to_string(),
            command_prefix: self.command_prefix.clone(),
            settings: self.settings.clone(),
            verification_role: roles.get(&RoleId(self.verification_role as u64)).map(|r| r.name.clone()),
            verified_role: roles.get(&RoleId(self.verified_role as u64)).map(|r| r.name.clone()),
            rankbinds,
            groupbinds,
            custombinds,
            assetbinds,
            blacklists: self.blacklists.clone()
        }
    }

    pub async fn from_backup(backup: BackupGuild, ctx: &Context, guild_id: GuildId, existing_roles: &Vec<Arc<CachedRole>>) -> Self {
        let mut names_to_ids = HashMap::<String, RoleId>::new();

        let all_roles = backup.rankbinds.iter()
            .flat_map(|r| r.discord_roles.iter().map(|r| r.clone()))
            .chain(backup.groupbinds.iter().flat_map(|g| g.discord_roles.iter().map(|r| r.clone())))
            .chain(backup.custombinds.iter().flat_map(|c| c.discord_roles.iter().map(|r| r.clone())))
            .chain(backup.assetbinds.iter().flat_map(|a| a.discord_roles.iter().map(|r| r.clone())))
            .unique()
            .collect::<Vec<String>>();
        for role_name in all_roles {
            if let Some(r) = existing_roles.iter().find(|r| r.name.eq_ignore_ascii_case(&role_name)) {
                names_to_ids.insert(role_name, r.id);
            } else {
                let role = ctx.http.create_role(guild_id).name(role_name).await.expect("Error creating a role");
                names_to_ids.insert(role.name, role.id);
            }
        }

        let rankbinds = backup.rankbinds.iter().map(|bind| RankBind::from_backup(bind, &names_to_ids)).collect_vec();
        let groupbinds = backup.groupbinds.iter().map(|bind| GroupBind::from_backup(bind, &names_to_ids)).collect_vec();
        let custombinds = backup.custombinds.iter().map(|bind| CustomBind::from_backup(bind, &names_to_ids)).collect_vec();
        let assetbinds = backup.assetbinds.iter().map(|bind| AssetBind::from_backup(bind, &names_to_ids)).collect_vec();

        let verification_role = if let Some(verification_name) = backup.verification_role {
            if let Some(r) = names_to_ids.get(&verification_name) {
                r.0 as i64
            } else if let Some(r) = existing_roles.iter().find(|e| e.name.eq(&verification_name)) {
                r.id.0 as i64
            } else {
                let role = ctx.http.create_role(guild_id).name(verification_name).await.expect("Error creating a role");
                role.id.0 as i64
            }
        } else {
            0
        };

        let verified_role = if let Some(verified_name) = backup.verified_role {
            if let Some(r) = names_to_ids.get(&verified_name) {
                r.0 as i64
            } else if let Some(r) = existing_roles.iter().find(|e| e.name.eq(&verified_name)) {
                r.id.0 as i64
            } else {
                let role = ctx.http.create_role(guild_id).name(verified_name).await.expect("Error creating a role");
                role.id.0 as i64
            }
        } else {
            0
        };

        Self {
            id: guild_id.0 as i64,
            command_prefix: backup.command_prefix,
            settings: backup.settings,
            verification_role,
            verified_role,
            rankbinds,
            groupbinds,
            custombinds,
            assetbinds,
            blacklists: backup.blacklists,
            disabled_channels: Vec::new()
        }
    }
}