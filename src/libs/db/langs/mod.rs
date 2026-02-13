// Relational
pub mod postgres;
pub mod mysql;
pub mod sqlite;
pub mod mariadb;
pub mod mssql;
pub mod oracle;

// Document
pub mod mongo;
pub mod couch;
pub mod dynamo;

// Key-value / Cache
pub mod redis;
pub mod valkey;
pub mod memcached;

// Search
pub mod elastic;
pub mod opensearch;

// Graph
pub mod neo4j;
pub mod dgraph;

// Time-series
pub mod influx;
pub mod timescale;

// Wide-column
pub mod cassandra;
pub mod scylla;

// Messaging / Streaming
pub mod kafka;
pub mod rabbitmq;
pub mod nats;
