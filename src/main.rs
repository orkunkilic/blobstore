pub mod indexer;
pub mod mempool;
pub mod network;
pub mod parser;

fn main() {
    let mempool = mempool::Mempool::new(None);
    let mempool = std::sync::Arc::new(mempool);
    let mut indexer = indexer::Indexer::new(
        mempool.clone(),
        "http://localhost:8332",
        "user",
        "password",
        None,
    );

    // Run in new thread
    tokio::spawn(async move {
        indexer.run().await;
    });

    tokio::spawn(async move {
        network::Behaviour::run(&mempool).await;
    });
}
