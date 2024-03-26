//--------------------------------------------------------------------------
// (C) Copyright Travis Sharp <travis@darkspace.dev>.  All rights reserved.
//--------------------------------------------------------------------------

use bson::Document;
use kuiperdb_core::error::Result;

pub mod expression;

/// A kuiper object (which is just a bson document)
pub type KuiperObject = Document;

/// A kuiper object iterator
pub type KuiperObjects = Box<dyn Iterator<Item = Result<KuiperObject>> + Send>;
