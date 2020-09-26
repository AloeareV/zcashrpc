fn make_client() -> zcashrpc::Client {
    let host = std::env::var("ZCASHRPC_TEST_HOST")
        .unwrap_or(zcashrpc::client::utils::get_zcashd_port());
    let auth = std::env::var("ZCASHRPC_TEST_AUTH")
        .or_else(get_cookie_with_env_var_err)
        .expect("cookie lookup failed");
    zcashrpc::Client::new(host, auth)
}

fn get_cookie_with_env_var_err(
    error: std::env::VarError,
) -> std::io::Result<String> {
    let log_msg = format!(
        "Invalid or no value passed to environment {} '{}'. {}",
        "variable ZCASHRPC_TEST_AUTH. Details:",
        error,
        "Defaulting to cookie lookup."
    );
    println!("{}", log_msg);

    cfg_if::cfg_if! {
        if #[cfg(feature = "cookie-finder")] {
            zcashrpc::client::utils::get_cookie(true)
        } else {
            let log_msg = format!(
                "For automatic authentication, run with flag {}",
                "'--features \"cookie-finder\"'."
            );
            panic!(log_msg);
        }
    }
}

macro_rules! run_smoketest {
    ($x:ident) => {
        #[tokio::test]
        async fn $x() {
            let _res = make_client().$x().await.unwrap();
        }
    };
}

run_smoketest!(getinfo);
run_smoketest!(getblockchaininfo);
