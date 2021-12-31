use bytes::BytesMut;
use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use serde::{Deserialize, Serialize};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::id::{
    ChannelId as DiscordChannelId, GuildId as DiscordGuildId, RoleId as DiscordRoleId,
    UserId as DiscordUserId,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct GuildId(pub DiscordGuildId);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct UserId(pub DiscordUserId);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RoleId(pub DiscordRoleId);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ChannelId(pub DiscordChannelId);

impl GuildId {
    pub fn new(n: u64) -> Self {
        Self(DiscordGuildId::new(n).unwrap())
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl UserId {
    pub fn new(n: u64) -> Self {
        Self(DiscordUserId::new(n).unwrap())
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl RoleId {
    pub fn new(n: u64) -> Self {
        Self(DiscordRoleId::new(n).unwrap())
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl ChannelId {
    pub fn new(n: u64) -> Self {
        Self(DiscordChannelId::new(n).unwrap())
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl Display for GuildId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl Display for RoleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl Display for ChannelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl ToSql for GuildId {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn StdError + Sync + Send>> {
        i64::to_sql(&(self.get() as i64), ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i64 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl ToSql for UserId {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn StdError + Sync + Send>> {
        i64::to_sql(&(self.get() as i64), ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i64 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl ToSql for RoleId {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn StdError + Sync + Send>> {
        i64::to_sql(&(self.get() as i64), ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i64 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl ToSql for ChannelId {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn StdError + Sync + Send>> {
        i64::to_sql(&(self.get() as i64), ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i64 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl<'a> FromSql<'a> for GuildId {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn StdError + Sync + Send>> {
        let id = i64::from_sql(ty, raw)?;
        Ok(Self::new(id as u64))
    }

    fn accepts(ty: &Type) -> bool {
        <i64 as FromSql>::accepts(ty)
    }
}

impl<'a> FromSql<'a> for UserId {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn StdError + Sync + Send>> {
        let id = i64::from_sql(ty, raw)?;
        Ok(Self::new(id as u64))
    }

    fn accepts(ty: &Type) -> bool {
        <i64 as FromSql>::accepts(ty)
    }
}

impl<'a> FromSql<'a> for RoleId {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn StdError + Sync + Send>> {
        let id = i64::from_sql(ty, raw)?;
        Ok(Self::new(id as u64))
    }

    fn accepts(ty: &Type) -> bool {
        <i64 as FromSql>::accepts(ty)
    }
}

impl<'a> FromSql<'a> for ChannelId {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn StdError + Sync + Send>> {
        let id = i64::from_sql(ty, raw)?;
        Ok(Self::new(id as u64))
    }

    fn accepts(ty: &Type) -> bool {
        <i64 as FromSql>::accepts(ty)
    }
}
