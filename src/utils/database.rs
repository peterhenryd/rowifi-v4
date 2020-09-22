use bson::{doc, Bson, document::Document};
use mongodb::{Client, options::*};
use futures::stream::StreamExt;
use crate::models::{
    guild::{RoGuild, BackupGuild}, 
    user::*
};
use super::error::RoError;

#[derive(Clone)]
pub struct Database {
    client: Client
}

impl Database {
    pub async fn new(conn_string: &str) -> Self {
        let client_options = ClientOptions::parse(conn_string).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        Self {
            client
        }
    }

    pub async fn add_guild(&self, guild: RoGuild, replace: bool) -> Result<(), RoError> {
        let guilds = self.client.database("RoWifi").collection("guilds");
        let guild_bson = bson::to_bson(&guild)?;
        if let Bson::Document(g) = guild_bson {
            if replace {
                let _ = guilds.find_one_and_replace(doc! {"_id": guild.id}, g, FindOneAndReplaceOptions::default()).await?;
            } else {
                let _ = guilds.insert_one(g, InsertOneOptions::default()).await?;
            }
        }
        Ok(())
    }

    pub async fn get_guild(&self, guild_id: u64) -> Result<Option<RoGuild>, RoError> {
        let guilds = self.client.database("RoWifi").collection("guilds");
        let result = guilds.find_one(doc! {"_id": guild_id}, FindOneOptions::default()).await?;
        match result {
            None => Ok(None),
            Some(res) => Ok(Some(bson::from_bson::<RoGuild>(Bson::Document(res))?))
        }
    }

    pub async fn get_guilds(&self, guild_ids: Vec<u64>, premium_only: bool) -> Result<Vec<RoGuild>, RoError> {
        let guilds = self.client.database("RoWifi").collection("guilds");
        let filter = match premium_only {
            true => doc! {"Settings.AutoDetection": true, "_id": {"$in": guild_ids}},
            false => doc! {"_id": {"$in": guild_ids}}
        };
        let mut cursor = guilds.find(filter, FindOptions::default()).await?;
        let mut result = Vec::<RoGuild>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => result.push(bson::from_bson::<RoGuild>(Bson::Document(document))?),
                Err(e) => return Err(e.into())
            }
        }
        Ok(result)
    }

    pub async fn modify_guild(&self, filter: Document, update: Document) -> Result<(), RoError> {
        let guilds = self.client.database("RoWifi").collection("guilds");
        let _res = guilds.update_one(filter, update, UpdateOptions::default()).await?;
        Ok(())
    }

    pub async fn add_queue_user(&self, user: QueueUser) -> Result<(), RoError> {
        let queue = self.client.database("RoWifi").collection("queue");

        let exists = queue.find_one(doc! {"_id": user.roblox_id}, FindOneOptions::default()).await?.is_some();

        let user_doc = bson::to_bson(&user)?;
        if let Bson::Document(u) = user_doc {
            if exists {
                let _ = queue.find_one_and_replace(doc! {"_id": user.roblox_id}, u, FindOneAndReplaceOptions::default()).await?;
            } else {
                let _ = queue.insert_one(u, InsertOneOptions::default()).await?;
            }
        }
        Ok(())
    }

    pub async fn add_user(&self, user: RoUser, verified: bool) -> Result<(), RoError> {
        let users = self.client.database("RoWifi").collection("users");
        let user_doc = bson::to_bson(&user)?;
        if let Bson::Document(u) = user_doc {
            if !verified {
                let _ = users.insert_one(u, InsertOneOptions::default()).await?;
            } else {
                let _ = users.find_one_and_replace(doc! {"_id": user.discord_id}, u, FindOneAndReplaceOptions::default()).await?;
            }
        }
        Ok(())
    }

    pub async fn get_user(&self, user_id: u64) -> Result<Option<RoUser>, RoError> {
        let users = self.client.database("RoWifi").collection("users");
        let result = users.find_one(doc! {"_id": user_id}, FindOneOptions::default()).await?;
        match result {
            None => Ok(None),
            Some(res) => Ok(Some(bson::from_bson::<RoUser>(Bson::Document(res))?))
        }
    }

    pub async fn get_users(&self, user_ids: Vec<u64>) -> Result<Vec<RoUser>, RoError> {
        let users = self.client.database("RoWifi").collection("users");
        let filter = doc! {"_id": {"$in": user_ids}};
        let mut cursor = users.find(filter, FindOptions::default()).await?;
        let mut result = Vec::<RoUser>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => result.push(bson::from_bson::<RoUser>(Bson::Document(document))?),
                Err(e) => return Err(e.into())
            }
        }
        Ok(result)
    }
    
    pub async fn add_backup(&self, mut backup: BackupGuild, name: &str) -> Result<(), RoError> {
        let backups = self.client.database("RoWifi").collection("backups");
        match self.get_backup(backup.user_id as u64, name).await? {
            Some(b) => {
                backup.id = b.id;
                let backup_bson = bson::to_bson(&backup)?;
                if let Bson::Document(b) = backup_bson {
                    let _ = backups.find_one_and_replace(doc! {"UserId": backup.user_id, "Name": backup.name}, b, FindOneAndReplaceOptions::default()).await?;
                }
            },
            None => {
                let backup_bson = bson::to_bson(&backup)?;
                if let Bson::Document(b) = backup_bson {
                    let _ = backups.insert_one(b, InsertOneOptions::default()).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn get_backup(&self, user_id: u64, name: &str) -> Result<Option<BackupGuild>, RoError> {
        let backups = self.client.database("RoWifi").collection("backups");
        let filter = doc! {"UserId": user_id, "Name": name};
        let result = backups.find_one(filter, FindOneOptions::default()).await?;
        match result {
            Some(b) => Ok(Some(bson::from_bson::<BackupGuild>(Bson::Document(b))?)),
            None => Ok(None)
        }
    }

    pub async fn get_backups(&self, user_id: u64) -> Result<Vec<BackupGuild>, RoError> {
        let backups = self.client.database("RoWifi").collection("backups");
        let filter = doc! {"UserId": user_id};
        let mut cursor = backups.find(filter, FindOptions::default()).await?;
        let mut result = Vec::<BackupGuild>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => result.push(bson::from_bson::<BackupGuild>(Bson::Document(document))?),
                Err(e) => return Err(e.into())
            }
        }
        Ok(result)
    }
}