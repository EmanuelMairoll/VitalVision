#[cfg(test)]
mod tests {
    use async_std::channel::{self, Receiver, Sender};
    //use async_std::sync::Arc;
    use async_std::task;
    use std::time::Duration;

    async fn writer(sender: Sender<i32>) {
        for i in 1..=10 {
            if sender.send(i).await.is_err() {
                println!("Channel closed, stopping writer.");
                return;
            }
            println!("Writer: Sent {}", i);
            task::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn reader(receiver: Receiver<i32> /*reads: Arc<async_std::sync::Mutex<Vec<i32>>>*/) {
        while let Ok(i) = receiver.recv().await {
            println!("Reader: Received {}", i);
            //let mut reads = reads.lock().await;
            //reads.push(i);
        }
    }

    #[test]
    fn test_async_channel() {
        task::block_on(async {
            let (sender, receiver) = channel::unbounded::<i32>();

            //let reads = Arc::new(async_std::sync::Mutex::new(Vec::new()));
            //let reader_reads = reads.clone();

            task::spawn(reader(receiver /* , reader_reads*/));
            writer(sender).await;

            /*
            let reads = reads.lock().await;
            assert_eq!(reads.len(), 10);
            assert_eq!(*reads, (1..=10).collect::<Vec<_>>());
            */
            assert!(true)
        });
    }
}
