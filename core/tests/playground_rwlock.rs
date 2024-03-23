use async_std::sync::RwLock;
use async_std::task;
use std::sync::Arc;
use std::time::Duration;

async fn writer(data: Arc<RwLock<i32>>) {
    for i in 1..=10 {
        {
            let mut data = data.write().await;
            *data = i;
            println!("Writer: Updated value to {}", i);
        }
        task::sleep(Duration::from_millis(100)).await;
    }
}

async fn reader(data: Arc<RwLock<i32>>, reads: Arc<RwLock<Vec<i32>>>) {
    for _ in 0..10 {
        {
            let data = data.read().await;
            reads.write().await.push(*data);
            println!("Reader: Current value is {}", *data);
        }
        task::sleep(Duration::from_millis(50)).await;
    }
}

#[test]
fn test_async_rwlock() {
    let data = Arc::new(RwLock::new(0));
    let reads = Arc::new(RwLock::new(Vec::new()));

    let writer_data = Arc::clone(&data);
    let reader_data = Arc::clone(&data);
    let reader_reads = Arc::clone(&reads);

    task::block_on(async {
        task::spawn(writer(writer_data));
        task::spawn(reader(reader_data, reader_reads)).await;
    });

    task::block_on(async {
        let reads = reads.read().await;
        assert!(
            !reads.is_empty(),
            "The reader should have read some values."
        );
        // Additional assertions can be added here based on your specific test requirements
    });
}
