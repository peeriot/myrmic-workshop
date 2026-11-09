#![no_std]
#![no_main]

use core::fmt::{Debug, Display};
use esp_firmware::embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp_firmware::embassy_sync::signal::Signal;
use esp_firmware::embassy_time::{Duration, Instant, Timer, with_timeout};
use esp_firmware::esp_hal::gpio::interconnect::{PeripheralInput, PeripheralOutput};
use esp_firmware::esp_hal::peripherals::{self, LEDC, Peripherals};
use esp_firmware::static_cell::StaticCell;
use esp_firmware::{Cell, Board, Network, NodeStatus, Sri, embassy_executor, embassy_time, log, status, status_watch};
use esp_firmware::esp_hal::ledc::timer::{self, TimerIFace};
use esp_firmware::esp_hal::ledc::channel::{self, ChannelHW, ChannelIFace};
use esp_firmware::esp_hal::{time::Rate, ledc::{self, LSGlobalClkSource, Ledc, LowSpeed}};
use esp_firmware::embassy_executor::{SpawnToken, Spawner};
use esp_firmware::esp_hal::gpio::{DriveMode, InputConfig, Input, InputPin, Pull};
use serde::{Serialize, Deserialize};mod status_led;

mod tank;
use tank::Command as Tank;

use crate::tank::Action;

const FAST_BLINK: Duration = Duration::from_millis(150);

const SLOW_BLINK: Duration = Duration::from_millis(600);
const DEBOUNCE: Duration = Duration::from_millis(20); static COMMAND_SIGNAL: Signal<CriticalSectionRawMutex, tank::Action> = Signal::new();pub struct StatusLed {
    channel: channel::Channel<'static, LowSpeed>,
    brightness: u32,}impl StatusLed {
    pub fn new(pin: impl PeripheralOutput<'static>, ledc: LEDC<'static>, brightness: u32,)-> StatusLed{
        let mut ledc = Ledc::new(ledc);
        ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);        
        
        static TIMER: StaticCell<timer::Timer<'static, LowSpeed>> = StaticCell::new();
        let timer = TIMER.init(ledc.timer::<LowSpeed>(timer::Number::Timer0));
        timer.configure(timer::config::Config{
            duty: timer::config::Duty::Duty10Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(20),
        }).unwrap();        let mut channel = ledc.channel(ledc::channel::Number::Channel0, pin);        channel.configure(channel::config::Config{
            timer: &*timer,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull
        }).unwrap();        
        channel.set_duty_hw(16);
        let brightness = brightness.min(1024u32);
        StatusLed { channel, brightness }       }       pub fn set(&self,on: bool){
        self.channel.set_duty_hw(if on {
            self.brightness} else {0}
        );
       }       
       
       pub fn spawn(self, spawner: Spawner){
        spawner.spawn(led_task(self).unwrap());       }
}

#[embassy_executor::task(embassy_executor = esp_firmware::embassy_executor)]
async fn led_task(led: StatusLed ){
    
    let mut watch = status_watch().expect("both status watchers already taken");
    let mut lit = true;
    let mut logged_status = None;    loop {
        let status = status();
        let (on, blink) = match status {
            NodeStatus::Down => (false, None),
            NodeStatus::Associated => (lit, Some(FAST_BLINK)),
            NodeStatus::Connected => (lit, Some(SLOW_BLINK)),
            NodeStatus::Registered => (true, None),
                    };                    if logged_status != Some(status) {
                        logged_status = Some(status);
                        log::info!("[status] {status:?}");
                    }                    led.set(on);                    
                    
                    
                    
                    let changed = match blink {
                        Some(half_period) => with_timeout(half_period,watch.changed()).await.is_ok(),
                        None => {
                            watch.changed().await;
                            true
                        }
                    };                    lit = changed || !lit;
                    
    }}
pub struct Button {
    pin: u8,
    input: Input<'static>,
    debounce: Duration
}
impl Button {    pub fn new(pin: impl InputPin + 'static, debounce: Duration ) -> Self {
        Self {
            pin: pin.number(),
            
            
            input: Input::new(pin, InputConfig::default().with_pull(Pull::Up)),
            debounce,
        }
    }    pub fn spawn(self, spawner: Spawner){
    
    spawner.spawn(button_task(self).unwrap());    }}

#[embassy_executor::task(embassy_executor = esp_firmware::embassy_executor)]
async fn button_task(mut button: Button){    loop {
        
        
        button.input.wait_for_low().await;
        Timer::after(button.debounce).await;
        if button.input.is_high(){
            continue;
        }
        let now = Instant::now();
        esp_firmware::log::info!("button at pin {} clicked @ {}", button.pin, now.as_millis());
        COMMAND_SIGNAL.signal(Action::Fire);
        
        button.input.wait_for_high().await;
        Timer::after(button.debounce).await;
    }}

impl Tank<'static> {
    pub fn new() -> Tank<'static> {
        Tank{
        player_name : "Tom",
        action : tank::Action::Join
        }
    }
    pub fn spawn_tank_task(self, spawner: Spawner, mut cell: Cell){
        spawner.spawn(tank_task(self,cell).unwrap());
    }    pub fn set_action(&mut self, action: Action){
        self.action = action
    }
}

#[embassy_executor::task(embassy_executor = esp_firmware::embassy_executor)]
async fn tank_task(mut tank: Tank<'static>, mut cell: Cell) {    
    
    let router_uuid = Sri::from_target("router").unwrap();    esp_firmware::log::info!("router sri {}", router_uuid);       
    
    
    
    
    let mut tx = cell.open().await;
    esp_firmware::log::info!("open tx");
    let _ = tx.send(router_uuid,"tank",  &tank).await;    if let Err(e) = tx.commit().await{
            log::warn!("tank command failed: {}", e);    }
    esp_firmware::log::info!("command sent and commited");    
    
    
    
    loop {
        let action  = COMMAND_SIGNAL.wait().await;
        tank.set_action(action);
            esp_firmware::log::info!("open tx");            let mut tx = cell.open().await;            let _ = tx.send(router_uuid,"tank",  &tank).await;
                if let Err(e) = tx.commit().await{
            log::warn!("tank command failed: {}", e);
    }    }
}

#[esp_firmware::main]
async fn setup(board: &mut Board, net: Network, peripherals: Peripherals, spawner: Spawner) {    let brightness = 16u32;
    let led_status = StatusLed::new(peripherals.GPIO7, peripherals.LEDC, brightness );
    let button = Button::new(peripherals.GPIO9, DEBOUNCE);    led_status.spawn(spawner);
    button.spawn(spawner);    
    
    
    
    let mut cell = match net.register("my-tank", &[]){
        Ok(cell) => cell,
        Err(err) => {
            esp_firmware::log::error!("unable to register cell: {err:?}");
            return;
        }
    };    let tank = Tank::new();    
    
    tank.spawn_tank_task(spawner, cell);}
