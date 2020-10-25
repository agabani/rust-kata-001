use redis::aio::MultiplexedConnection;

pub(crate) async fn new(uri: &str) -> Result<MultiplexedConnection, String> {
    let fn_name = "new";

    redis::Client::open(uri)
        .map_err(|error| {
            log::error!("{}: error={:?}", fn_name, error);
            format!("{}: error={:?}", fn_name, error)
        })?
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|error| {
            log::error!("{}: error={:?}", fn_name, error);
            format!("{}: error={:?}", fn_name, error)
        })
}
