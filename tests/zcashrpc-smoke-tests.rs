macro_rules! run_smoketest {
    ($x:ident) => {
        #[tokio::test]
        async fn $x() {
            let _res = zcashrpc::client::utils::make_client(true).$x().await.unwrap();
        }
    };
}

run_smoketest!(getinfo);
run_smoketest!(getblockchaininfo);
