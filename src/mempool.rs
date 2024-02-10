// Mempool to store gossipped blobs

pub struct Mempool {
    db: rocksdb::DB,
}

impl Mempool {
    pub fn new(db_path: Option<&str>) -> Self {
        let db = match db_path {
            Some(path) => rocksdb::DB::open_default(path).unwrap(),
            None => rocksdb::DB::open_default("mempool-db").unwrap(),
        };

        Mempool { db }
    }

    pub fn add_blob(&self, hash: &[u8], blob: &[u8], _timestamp: u64) {
        // TODO: Add timestamp to blob
        self.db.put(hash, blob).unwrap();
    }

    pub fn get_blob(&self, hash: &[u8]) -> Option<Vec<u8>> {
        self.db.get(hash).unwrap()
    }

    pub fn remove_blob(&self, hash: &[u8]) {
        self.db.delete(hash).unwrap();
    }

    pub fn prune(&self) {
        // TODO: Prune blobs older than X time
    }
}
