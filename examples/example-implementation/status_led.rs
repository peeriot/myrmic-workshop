
use esp_firmware::esp_hal::gpio::DriveMode;
use esp_firmware::esp_hal::gpio::interconnect::PeripheralOutput;
use esp_firmware::esp_hal::ledc::channel::{self, ChannelHW, ChannelIFace};
use esp_firmware::esp_hal::ledc::timer::{self, TimerIFace};
use esp_firmware::esp_hal::ledc::{LSGlobalClkSource, Ledc, LowSpeed};
use esp_firmware::esp_hal::time::Rate;
use esp_firmware::{
    NodeStatus, embassy_executor, embassy_time, log, static_cell, status, status_watch,
};
use esp_firmware::esp_hal::peripherals::LEDC;
use embassy_executor::Spawner;
use embassy_time::{Duration, with_timeout};
use static_cell::StaticCell;
const FAST_BLINK: Duration = Duration::from_millis(150);const SLOW_BLINK: Duration = Duration::from_millis(600);pub struct StatusLed {
    channel: channel::Channel<'static, LowSpeed>,
    brightness: u32,
}
impl StatusLed {
    
    pub fn new(ledc: LEDC<'static>, pin: impl PeripheralOutput<'static>, brightness: u32) -> Self {
        let mut ledc = Ledc::new(ledc);
        ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);        
        static TIMER: StaticCell<timer::Timer<'static, LowSpeed>> = StaticCell::new();
        let timer = TIMER.init(ledc.timer(timer::Number::Timer0));
        timer
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty10Bit,
                clock_source: timer::LSClockSource::APBClk,
                
                frequency: Rate::from_khz(20),
            })
            .unwrap();        let mut channel = ledc.channel(channel::Number::Channel0, pin);
        channel
            .configure(channel::config::Config {
                timer: &*timer,
                duty_pct: 0,
                drive_mode: DriveMode::PushPull,
            })
            .unwrap();        let brightness = brightness.min(1024u32);        Self {
            channel,
            brightness,
        }
    }    fn set(&self, on: bool) {
        self.channel
            .set_duty_hw(if on { self.brightness } else { 0 });
    }    
    pub fn spawn(self, spawner: Spawner) {
        spawner.spawn(task(self).unwrap());
    }
}

#[embassy_executor::task(embassy_executor = esp_firmware::embassy_executor)]
async fn task(led: StatusLed) {
    let mut watch = status_watch().expect("both status watchers already taken");
    let mut lit = true;
    let mut said = None;    loop {
        
        
        let status = status();
        let (on, blink) = match status {
            NodeStatus::Down => (false, None),
            NodeStatus::Associated => (lit, Some(FAST_BLINK)),
            NodeStatus::Connected => (lit, Some(SLOW_BLINK)),
            NodeStatus::Registered => (true, None),
        };  
        
        if said != Some(status) {
            said = Some(status);
            log::info!("[status] {status:?}");
        }
        led.set(on);        
        let changed = match blink {
            Some(half_period) => with_timeout(half_period, watch.changed()).await.is_ok(),
            None => {
                watch.changed().await;
                true
            }
        };
        lit = changed || !lit;
    }
}
