//--------------------------------------------------------------------------
// (C) Copyright Travis Sharp <travis@darkspace.dev>.  All rights reserved.
//--------------------------------------------------------------------------

// SCHEMA:      4 bytes
// Padding:     2 bytes
// COLLECTION:  4 bytes
// Padding:     2 bytes
// UUID:        16-bytes
// TOTAL:       28 bytes

// ID STORAGE:  ~28 GB / 1b Records

// 65535 max schemas
pub const INFORMATION_SCHEMA: u32 = 0x0000;
pub const DEFAULT_SCHEMA: u32 = 0x0001;

// 65535 max tables / schema
// Tables
pub const SCHEMA: u32 = 0x0000;
pub const TABLE: u32 = 0x0001;
pub const TABLE_PRIVILIGES: u32 = 0x0002;
pub const REFERENTIAL_CONSTRAINTS: u32 = 0x0002;
pub const CHECK_CONSTRAINTS: u32 = 0x0002;
pub const TABLE_CONSTRAINTS: u32 = 0x0002;

pub const PADDING: [u8; 2] = *b"::";
