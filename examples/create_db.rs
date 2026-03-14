use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let database_url = "sqlite:template.db";

    println!("Creating database at: {}", database_url);

    let pool = SqlitePool::connect(database_url).await?;

    println!("Creating files table...");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            hash INTEGER NOT NULL,
            modified_at INTEGER NOT NULL,
            parent_id INTEGER
        )
        "#,
    )
    .execute(&pool)
    .await?;

    println!("Database initialized successfully!");
    println!("Table 'files' created with columns:");
    println!("  - id (INTEGER PRIMARY KEY)");
    println!("  - name (TEXT NOT NULL)");
    println!("  - hash (INTEGER NOT NULL)");
    println!("  - modified_at (INTEGER NOT NULL)");
    println!("  - parent_id (INTEGER)");

    Ok(())
}
