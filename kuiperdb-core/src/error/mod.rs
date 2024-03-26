//--------------------------------------------------------------------------
// (C) Copyright Travis Sharp <travis@darkspace.dev>.  All rights reserved.
//--------------------------------------------------------------------------

use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

/// Result returning Error
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Error, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Error {
    /// There was a problem with a datastore transaction
    #[error("There was a problem with a datastore transaction: {0}")]
    Tx(String),

    /// There was an error when starting a new datastore transaction
    #[error("There was an error when starting a new datastore transaction")]
    TxFailure,

    /// The transaction was already cancelled or committed
    #[error("Couldn't update a finished transaction")]
    TxFinished,

    /// The current transaction was created as read-only
    #[error("Couldn't write to a read only transaction")]
    TxReadonly,

    /// The conditional value in the request was not equal
    #[error("Value being checked was not correct")]
    TxConditionNotMet,

    /// The key being inserted in the transaction already exists
    #[error("The key being inserted already exists")]
    TxKeyAlreadyExists,

    #[error("{0}")]
    Parse(String),

    #[error("{0}")]
    Value(String),
}

impl From<rocksdb::Error> for Error {
    fn from(e: rocksdb::Error) -> Error {
        Error::Tx(e.to_string())
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::Value(err.to_string())
    }
}
