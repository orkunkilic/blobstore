use std::sync::Arc;

use bitcoincore_rpc::{Auth, Client, RpcApi};
use rocksdb::DB;

use crate::{mempool::Mempool, parser::find_pattern_instructions};

pub struct Indexer {
    mempool: Arc<Mempool>,
    db: DB,
    client: Client,
    last_block: u64,
}

impl Indexer {
    pub fn new(
        mempool: Arc<Mempool>,
        rpc_url: &str,
        rpc_user: &str,
        rpc_pass: &str,
        db_path: Option<&str>,
    ) -> Self {
        let db = match db_path {
            Some(path) => DB::open_default(path).unwrap(),
            None => DB::open_default("indexer-db").unwrap(),
        };

        let last_block = match db.get("last_block") {
            Ok(Some(value)) => {
                let value = String::from_utf8(value).unwrap();
                value.parse::<u64>().unwrap()
            }
            _ => 0,
        };

        let client = Client::new(
            rpc_url,
            Auth::UserPass(rpc_user.to_string(), rpc_pass.to_string()),
        )
        .unwrap();

        Indexer {
            mempool,
            db,
            client,
            last_block,
        }
    }

    fn index_block(&mut self, block: &bitcoin::Block) {
        let mut block_blobs = vec![];

        for tx in &block.txdata {
            // Get tx witness data
            for input in &tx.input {
                let mut instructions = match input.witness.tapscript() {
                    Some(script) => script.instructions().peekable(),
                    None => continue,
                };

                let (hash, size) = match find_pattern_instructions(&mut instructions) {
                    Some((hash, size)) => (hash, size),
                    None => continue,
                };

                // Check mempool if hash exists
                match self.mempool.get_blob(&hash) {
                    Some(data) => {
                        // Remove from mempool
                        self.mempool.remove_blob(&hash);

                        // Check if size matches
                        if data != size {
                            continue;
                        }

                        // Add to block blobs
                        block_blobs.push((hash, data));
                    }
                    _ => continue,
                };
            }
        }

        // TODO: 2D Reed-Solomon encoding over blobs
        // TODO: broadcast block blobs to network

        // Store blobs in db
        for (hash, data) in block_blobs {
            self.db.put(&hash, data).unwrap();
        }
    }

    pub async fn run(&mut self) {
        while let Ok(height) = self.client.get_block_count() {
            if height > self.last_block {
                for block_height in self.last_block..height {
                    let block_hash = self.client.get_block_hash(block_height).unwrap();
                    let block = self.client.get_block(&block_hash).unwrap();
                    self.index_block(&block);
                }
                self.last_block = height;
                self.db.put("last_block", height.to_string()).unwrap();
                self.last_block = height;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        }
    }
}
