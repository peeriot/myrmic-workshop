#[esp_firmware::embassy_executor::task(embassy_executor = esp_firmware::embassy_executor)]
async fn button_handler(mut button: Input<'static>) {
    const DEBOUNCE: Duration = Duration::from_millis(20);
            
    loop {
        button.wait_for_low().await;
        Timer::after(DEBOUNCE).await;
        if button.is_high() {
            continue;
        }
        let now = Instant::now();
        esp_firmware::log::info!("[button] clicked @ {}", now.as_millis());

        button.wait_for_high().await;
        Timer::after(DEBOUNCE).await;
    }
}
