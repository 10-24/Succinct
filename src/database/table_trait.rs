use redb::{
    Key, ReadOnlyTable, ReadTransaction, ReadableTable , Table as RedbTable, TableDefinition, TableError, Value, WriteTransaction
};

pub type TableConst<T> = ReadOnlyTable<<T as TableDef>::Key, <T as TableDef>::Value>;
pub trait ReadTable<T: TableDef>: ReadableTable<T::Key, T::Value> {}
impl<T: TableDef, X: ReadableTable<T::Key, T::Value>> ReadTable<T> for X {}
pub type TableMut<'txn, T> = RedbTable<'txn, <T as TableDef>::Key, <T as TableDef>::Value>;

pub trait TableDef {
    type Key: Key + 'static;
    type Value: Value + 'static;

    const NAME: &'static str;
    const DEF: TableDefinition<'static, Self::Key, Self::Value> = TableDefinition::new(Self::NAME);
}

pub trait Txn {
    type Out<'txn, K: Key + 'static, V: Value + 'static>
    where
        Self: 'txn;

    fn open<'txn, T:TableDef>(
        &'txn self,
    ) -> Result<Self::Out<'txn, T::Key, T::Value>, TableError>;

}

impl Txn for ReadTransaction {
    type Out<'txn, K: Key + 'static, V: Value + 'static> = ReadOnlyTable<K, V>
    where
        Self: 'txn;

    fn open<'txn, T:TableDef>(
        &'txn self,
    ) -> Result<TableConst<T>, TableError> {
        self.open_table(T::DEF)
    }
}

impl Txn for WriteTransaction {
    type Out<'txn, K: Key + 'static, V: Value + 'static> = RedbTable<'txn, K, V>
    where
        Self: 'txn;

    fn open<'txn, T:TableDef>(
        &'txn self,
    ) -> Result<TableMut<T>, TableError> {
        self.open_table(T::DEF)
    }
}