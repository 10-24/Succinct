
use redb::{
    Key, MultimapTable, MultimapTableDefinition, ReadOnlyMultimapTable, ReadOnlyTable, ReadTransaction, Table, TableDefinition, Value, WriteTransaction
};

// --- TRAITS (Keep these public so the macro can see them) ---

pub trait OpenReadTable {
    type Out;
    fn open_table(self, txn: &ReadTransaction) -> Self::Out;
}

impl<'a, K: Key + 'static, V: Value + 'static> OpenReadTable for TableDefinition<'a, K, V> {
    type Out = ReadOnlyTable<K, V>;
    fn open_table(self, txn: &ReadTransaction) -> Self::Out {
        txn.open_table(self).unwrap()
    }
}

impl<'a, K: Key + 'static, V: Key + Value + 'static> OpenReadTable for MultimapTableDefinition<'a, K, V> {
    type Out = ReadOnlyMultimapTable<K, V>;
    fn open_table(self, txn: &ReadTransaction) -> Self::Out {
        txn.open_multimap_table(self).unwrap()
    }
}

pub trait OpenWriteTable<'a> {
    type Out;
    fn open_table_mut(self, txn: &'a WriteTransaction) -> Self::Out;
}

impl<'a, 'b, K: Key + 'static, V: Value + 'static> OpenWriteTable<'a> for TableDefinition<'b, K, V> {
    type Out = Table<'a, K, V>;
    fn open_table_mut(self, txn: &'a WriteTransaction) -> Self::Out {
        txn.open_table(self).unwrap()
    }
}
impl<'a, K: Key + 'static, V: Key + Value + 'static> OpenWriteTable<'a> for MultimapTableDefinition<'a, K, V> {
    type Out = MultimapTable<'a, K, V>;
    fn open_table_mut(self, txn: &'a WriteTransaction) -> Self::Out {
        txn.open_multimap_table(self).unwrap()
    }
}

// --- MACROS ---

#[macro_export]
macro_rules! tables {
    ($db:expr, $($table:expr),+) => {{ 
        let txn = $db.begin_read();
        (
            $(
                {
                    $crate::db::tables::macros::OpenReadTable::open_table($table, &txn)
                }
            ),+
        )
    }};
}

#[macro_export]
macro_rules! tables_mut {
    ($writer:expr, $($table:expr),+) => {{    
        (
            $(
                {
                    $crate::db::tables::macros::OpenWriteTable::open_table_mut($table, &$writer.txn)
                }
            ),+
        )
    }};
}