use mongodb::{
    bson::{doc, Bson},
    sync::Client,
};
use mongodb::sync::{Database, Collection};

    pub fn getDatabase()-> Database {
        let client : Client = Client::with_uri_str("mongodb://localhost:27017").unwrap();
        client.database("stocks")
    }
