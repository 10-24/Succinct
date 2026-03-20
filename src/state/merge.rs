use sqlx::{Acquire, PgConnection, SqliteConnection};

use crate::state::{local::{self, QueuedUpdate}, remote, state::State};

impl State {
    
    async fn clear_queue(&mut self, conn: &mut Biconn,) -> sqlx::Result<()> {
      
        let mut result = Ok(());
        let mut remote_txn = conn.remote.begin().await?;
        
        let deletion_records = local::get_delete_queue(&mut conn.local).await?;
        for record in deletion_records {
            if let Err(e) = remote::delete_file(&mut remote_txn, record.file_id, record.deleted_at).await {
                result = Err(e);
                continue;
            }
            local::remove_from_delete_queue(&mut conn.local, record.file_id).await.unwrap(); // IDK how to handle an err here
        }
        
        let update_records = local::get_update_queue(&mut conn.local).await?;
        for record in update_records {
            let record_id = record.id;
            if let Err(e) = remote::insert_file(&mut remote_txn, record).await {
                result = Err(e);
                continue;
            }
            local::remove_from_update_queue(&mut conn.local, record_id).await.unwrap() // IDK how to handle an err here
        }
        
        remote_txn.commit().await.and(result)
        
    }
    
    async fn push_update(&self, conn: &mut Biconn, update: QueuedUpdate) -> anyhow::Result<()> {
        let file_path = local::get_file_path(&mut conn.local, update.file_id).await?;
        let file_abs_path = self.local_root.join(&file_path);
        let is_file = tokio::fs::metadata(file_abs_path.as_ref()).await?.is_file();
        if !is_file {
            return Ok(());
        }
        let file_content = tokio::fs::read(file_abs_path.as_ref()).await?;
        self.remote_drive.write(file_path.as_ref(), file_content).await?;
        Ok(())
    }
    
}

pub struct Biconn {
    local: SqliteConnection, 
    remote: PgConnection
}