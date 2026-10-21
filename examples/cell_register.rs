use esp_firmware::{Network, Sri};

async fn register(net: Network) {
    let mut cell = match net.register("some-unique-srn", &[]) {
        Ok(cell) => cell,
        Err(err) => {
            esp_firmware::log::error!("unable to register cell: {err:?}");
            return;
        }
    };

    let router = Sri::from_target("router").unwrap();

    let payload = [0xde, 0xad, 0xbe, 0xef].to_vec();

    let mut tx = cell.open().await;

    let _ = tx.send_command(router, "tank", payload).await;

    let _ = tx.commit().await;
}
