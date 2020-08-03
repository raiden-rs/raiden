#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone)]
    pub struct User {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[test]
    fn test_minimum_transact_write() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let tx = ::raiden::WriteTx::new(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = User::condition().attr_not_exists(User::id());
            let input = User::put_item_builder()
                .id("testId".to_owned())
                .name("bokuweb".to_owned())
                .build();
            let input2 = User::put_item_builder()
                .id("testId2".to_owned())
                .name("bokuweb".to_owned())
                .build();
            assert_eq!(
                tx.put(User::put(input).condition(cond))
                    .put(User::put(input2))
                    .run()
                    .await
                    .is_ok(),
                true,
            )
        }
        rt.block_on(example());
    }

    #[test]
    fn test_transact_write_put_and_update() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let tx = ::raiden::WriteTx::new(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let input = User::put_item_builder()
                .id("testId".to_owned())
                .name("bokuweb".to_owned())
                .build();
            let set_expression = User::update_expression()
                .set(User::name())
                .value("updated!!");

            let res = tx
                .put(User::put(input))
                .update(User::update("testId2").set(set_expression))
                .run()
                .await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }

    #[test]
    fn test_transact_write_with_prefix_suffix() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let tx = ::raiden::WriteTx::new(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let input = User::put_item_builder()
                .id("testId".to_owned())
                .name("bokuweb".to_owned())
                .build();
            assert_eq!(
                tx.put(
                    User::put(input)
                        .table_prefix("test-")
                        .table_suffix("-staging"),
                )
                .run()
                .await
                .is_ok(),
                true,
            )
        }
        rt.block_on(example());
    }

    use std::sync::atomic::{AtomicUsize, Ordering};

    static RETRY_COUNT: AtomicUsize = AtomicUsize::new(1);
    struct MyRetryStrategy;

    impl RetryStrategy for MyRetryStrategy {
        fn should_retry(&self, _error: &RaidenError, retry_count: usize) -> bool {
            RETRY_COUNT.store(retry_count, Ordering::Relaxed);
            true
        }

        fn policy(&self) -> Policy {
            Policy::Limit(3)
        }
    }

    #[test]
    fn test_retry() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let tx = ::raiden::WriteTx::new(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let input = User::put_item_builder()
                .id("testId".to_owned())
                .name("bokuweb".to_owned())
                .build();
            assert_eq!(
                tx.with_retries(Box::new(MyRetryStrategy))
                    .put(User::put(input).table_prefix("unknown"))
                    .run()
                    .await
                    .is_err(),
                true,
            )
        }
        rt.block_on(example());
        assert_eq!(RETRY_COUNT.load(Ordering::Relaxed), 3)
    }
}
