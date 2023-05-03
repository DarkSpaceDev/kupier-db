// For Index Implementation:  https://github.com/facebook/rocksdb/wiki/Column-Families
// Relational DB Example:     https://blog.petitviolet.net/post/2021-05-25/building-database-on-top-of-rocksdb-in-rust

pub struct Database {
    pub name: String,
    pub tables: Vec<Table>,
}

pub struct Table {
    pub name: String,
    pub indexes: Vec<Index>,
    pub constraints: Vec<Constraint>,
}

pub struct Constraint {}

pub enum IndexType {
    NON_CLUSTERED,
    CLUSTERED,
    UNIQUE,
}

pub struct Index {
    pub name: String,
    pub index_type: IndexType,
}
