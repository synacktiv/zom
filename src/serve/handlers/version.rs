pub async fn version_handler() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
