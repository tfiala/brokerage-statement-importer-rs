use anyhow::Result;
use async_trait::async_trait;
use mongodb::{bson, Database};
use serde_derive::{Deserialize, Serialize};


use mongodb_migrator::migration::Migration;

async fn run_migrations(db: &mongodb::Database) -> Result<()> {
    let migrations: Vec<Box<dyn Migration>> = vec![Box::new(M0 {}), Box::new(M1 {})];
    mongodb_migrator::migrator::DefaultMigrator::new()
        .with_conn(db.clone())
        .with_migrations_vec(migrations)
        .up()
        .await?;

    Ok(())
}

struct M0 {}
struct M1 {}

#[async_trait]
impl Migration for M0 {
    async fn up(&self, db: Database) -> Result<()> {
        db.collection("users")
            .insert_one(bson::doc! { "name": "Batman" }, None)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl Migration for M1 {
    async fn up(&self, db: Database) -> Result<()> {
        db.collection::<Users>("users")
            .update_one(
                bson::doc! { "name": "Batman" },
                bson::doc! { "$set": { "name": "Superman" } },
                None,
            )
            .await?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Users {
    name: String,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use mongodb::{Client, Database};
    use testcontainers::Docker;
    use tokio::test;
    use super::*;

    #[tokio::test]
    async fn test_migrations_run() -> Result<()> {
        let docker = testcontainers::clients::Cli::default();
        let node = docker.run(testcontainers::images::mongo::Mongo::default());
        let host_port = node.get_host_port(27017).unwrap();
        let url = format!("mongodb://localhost:{}/", host_port);
        let client = mongodb::Client::with_uri_str(url).await.unwrap();
        let db = client.database("test");

        run_migrations(&db).await?;

        Ok(())
    }
}
