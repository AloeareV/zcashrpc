macro_rules! run_smoketest {
    ($x:ident) => {
        #[tokio::test]
        async fn $x() {
            let _res = zcashrpc::client::utils::make_client(true)
                .$x()
                .await
                .unwrap();
        }
    };
}

run_smoketest!(getinfo);
run_smoketest!(getblockchaininfo);

#[test]
fn serialize_and_deserialize_empty() {
    let foo = "";
    let jfoo = serde_json::json!(foo);
    assert_eq!(foo, jfoo);
    assert_eq!(format!("{}", jfoo), format!("{}", foo));
}
