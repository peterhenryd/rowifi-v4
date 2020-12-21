pub use mongodb::{
    bson::{de::Error as DeserializationError, ser::Error as SerializationError},
    error::Error as MongoError,
};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub enum DatabaseError {
    Serialization(SerializationError),
    Deserialization(DeserializationError),
    Mongo(MongoError),
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            DatabaseError::Serialization(err) => write!(f, "Serialization Error - {}", err),
            DatabaseError::Deserialization(err) => write!(f, "Deserialization Error - {}", err),
            DatabaseError::Mongo(err) => write!(f, "Mongo Error - {}", err),
        }
    }
}

impl From<SerializationError> for DatabaseError {
    fn from(err: SerializationError) -> Self {
        DatabaseError::Serialization(err)
    }
}

impl From<DeserializationError> for DatabaseError {
    fn from(err: DeserializationError) -> Self {
        DatabaseError::Deserialization(err)
    }
}

impl From<MongoError> for DatabaseError {
    fn from(err: MongoError) -> Self {
        DatabaseError::Mongo(err)
    }
}

impl StdError for DatabaseError {}