use fjall::Database;

use crate::context::Context;






mod constants;
mod context;
mod db;
mod delta;
mod util;
mod notify;
/*
 * Unimplemented Features:
 * Handle if you update a child then delete the parent.
 * Fix Root Id
 * Watch new files
 * Error Notifications
 * Add and remove ignored file from the command line
 * Add check in walk dir for uninitialized files
 * Graceful shutdown
 */
mod walk_dir;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let ctx = Context::load().await?;
    
    // Attach listeners
    loop {

        
        // Pull changes from remote DB, excluding those that collide with local changes
    
        // Push queued changes to opendal
    
        // Update remote db


        // Aggregate File changes
        // Save queued changes to local_db
    }

   
}


