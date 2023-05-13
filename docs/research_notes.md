# Architecture
![](./assets/architecture.png)

# Feature List v1
* (p0.2) Interface: Wire Protocol - http
* (p0.1) Engine: Language Parser (SQLish)
* (p0.0) Engine: Query Planner & Executor
* Document Relationships
* Synthetic Tables ...
* Storage: [*] Txn
* Storage: [*] KV Store
* Storage: [*] Index Support - (Column Families)
* Storage: [] Backups (Checkpoints)
* Storage: VFS Single File?
* Storage: At Rest Encryption
* Storage: Field Level Encryption
* Storage: Document Level Encryption

# KV Store Selection
While yes RocksDb is a highly-performant kv store - it's selection is due to these factors:
* ACID Transaction Support
* Plug-In Capable to build in more advanced features
* High-Performance Key Lookups (point reads)
* The construct of a column-family which can be directly leveraged for indexing support
* COTS LSM / Bloom Filtering that was first assesed in the original research for the v1 work

# Databases
If you think of a database server as in the context of MongoDb, MsSQL, Postgres, etc. then you will undoubtedly understand that there is the idea of a 'database' which can either be a single file or a collection of files that comprise the database. The database engine represents a single siloed database. All tables, records, transactions, etc. live within the scope of that single database. Multi-database transactions are not a goal of this implementation detail - however - a more general database management system in the future will be. This would allowed shared use of resources; however, you will need to keep in mind resource contention when leveraging multi-database implementations.

# Tables (Synthetic Tables)
Since RocksDB does not have the idea or construct of what a `Table` is - we construct one by leveraging key-prefixes. The first 2 bytes of each key determines a table namespace. The second 2 bytes determine the table identifier, which is stored in special system tables. The combination of 4 bytes is intentional as it represents 64-bit integer if combined.

Current implementation details will leave it as u64, or unsigned 64 and each part as unsigned 32 bit.

There will be a maximum of 4294967295 namespaces.
There will be a maximum of 4294967295 tables.

A 32 bit value would conserve space; but the prefix is being optimized for 64 bit architectures.
A table key is a UUID value.

A guid is 128 bits (2x64) and the NS+TB identifier is 64 bits which means this is approx 3 mov and 3 compare operators wheras using values that do not align to this would most likely cause performance hits when dealing with moving data around at this level ...

TODO: Find out the performance statistics when reading and writing data with various key lengths.
- Database Size
- Key Lengths that are impedance mismatched with the computer architecture
- Key Lengths that are impedance matched with the computer architecture.
- Key Lengths of 4x64-bit
- Key Lengths of 3x64-bit
- Key Lengths of 5x32-bit
Hypothesis - key-lengths that align better with the computer architecutre / cpu support will have better performance.

| Namespace            | Namespace Value   |
| -------------------- | ----------------- |
| System Namespace     | 0xFFFFFFFF        |
| User Namespace       | 0x00000000        |

*NOTE: This is similar to INFORMATION_SCHEMA in ANSI SQL.
| System Table Name       | System Table Value |
| ----------------------- | ------------------ |
| NAMESPACE               | 0x00000000         |
| TABLE                   | 0x00000001         |
| TABLE_CONSTRAINTS       | 0x00000002         |
| TABLE_PRIVILIGES        | 0x00000003         |
| TABLE_TRIGGERS          | 0x00000004         |
| REFERENTIAL_CONSTRAINTS | 0x00000005         |
| CHECK_CONSTRAINTS       | 0x00000006         |

# References
https://github.com/wspeirs/btree
https://yetanotherdevblog.com/bloom-filters/
https://yetanotherdevblog.com/dense-vs-sparse-indexes/
https://yetanotherdevblog.com/lsm/
https://github.com/petitviolet/rrrdb
https://github.com/surrealdb/surrealdb
https://github.com/sqlparser-rs/sqlparser-rs

# Current
So ... it seems RocksDb is a great fit for starting point as it provides transactions and a fantastic
database model to start. It uses Bloom Filters, LSM's, transactions, pluggable architecture, etc. This
will be a great starting point as it takes a lot of the initial storage guesswork out. For now - there
will need to be a few things required:
- Execution Planner
  - This will need indexing later but MUST support scanning.
- Query Language (I would like to use something similar to Kusto Query Language if Possible)
- MongoDb would be interesting...
- Choose data storage type. BSON is interesting but what else is there?
- BSON Spec is OpenSource ... so BSON it is!
