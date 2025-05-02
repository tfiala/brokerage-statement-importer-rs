use anydb::Result;
use mongodb::{Client, Database};

pub struct MongoClient {
    pub client: Client,
}

impl MongoClient {
    pub async fn new(uri: &str) -> Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        Ok(MongoClient { client })
    }

    pub async fn get_database(&self, db_name: &str) -> Database {
        self.client.database(db_name)
    }
}

impl Drop for MongoClient {
    fn drop(&mut self) {
        // Close the client connection when the struct is dropped
        let _ = self.client.close(None);
    }
}