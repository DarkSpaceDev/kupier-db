use bson::Document;
use kupier_core::error::Result;

pub mod expression;

/// A kupier object (which is just a bson document)
pub type KupierObject = Document;

/// A kupier object iterator
pub type KupierObjects = Box<dyn Iterator<Item = Result<KupierObject>> + Send>;
