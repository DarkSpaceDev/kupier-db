use bson::Document;
use kuiper_core::error::Result;

pub mod expression;

/// A kuiper object (which is just a bson document)
pub type kuiperObject = Document;

/// A kuiper object iterator
pub type kuiperObjects = Box<dyn Iterator<Item = Result<kuiperObject>> + Send>;
