pub struct RemoteDb {
    pub conn: redb::Database
}
impl RemoteDb {
    pub fn begin_write(&self) -> RemoteDbTxn {
        RemoteDbTxn {
            txn: self.conn.begin_write().unwrap(),
        }
    }
}

pub struct RemoteDbTxn {
    pub txn: redb::WriteTransaction,
}
impl RemoteDbTxn {
    pub fn commit(self) {
        self.txn.commit().unwrap();
    }
}