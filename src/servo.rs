use embassy_time::Timer;
use embedded_hal::pwm::SetDutyCycle;
use esp_hal::gpio::GpioPin;
use esp_hal::ledc::channel;
use esp_hal::prelude::*;
use esp_hal::{
    ledc::{timer, HighSpeed, Ledc},
    prelude::_esp_hal_ledc_timer_TimerIFace,
};

pub struct Motor<'a> {
    channel: channel::Channel<'a, HighSpeed>,
    duty_gap: u32,
    min_duty: u32,
}

pub const SERVO_PIN: u8 = 27;

impl<'a> Motor<'a> {
    pub fn new(ledc: Ledc<'static>, servo: GpioPin<SERVO_PIN>) -> Self {
        // let mut hstimer0 = ledc.timer::<HighSpeed>(timer::Number::Timer0);
        let hstimer0 = super::mk_static!(
            timer::Timer<'_, HighSpeed>,
            ledc.timer::<HighSpeed>(timer::Number::Timer0)
        );
        hstimer0
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty12Bit,
                clock_source: timer::HSClockSource::APBClk,
                frequency: 50.Hz(),
            })
            .unwrap();
        let mut channel = ledc.channel(channel::Number::Channel0, servo);
        channel
            .configure(channel::config::Config {
                timer: hstimer0,
                duty_pct: 12,
                pin_config: channel::config::PinConfig::PushPull,
            })
            .unwrap();

        let max_duty_cycle = channel.max_duty_cycle() as u32;

        // Minimum duty (2.5%)
        // For 12bit -> 25 * 4096 /1000 => ~ 102
        let min_duty = (25 * max_duty_cycle) / 1000;
        // Maximum duty (12.5%)
        // For 12bit -> 125 * 4096 /1000 => 512
        let max_duty = (125 * max_duty_cycle) / 1000;
        // 512 - 102 => 410
        let duty_gap = max_duty - min_duty;

        Self {
            channel,
            duty_gap,
            min_duty,
        }
    }

    pub async fn open_door(&mut self) {
        let duty = duty_from_angle(90, self.min_duty, self.duty_gap);
        self.channel.set_duty_cycle(duty).unwrap();
        Timer::after_millis(20).await;
    }

    pub async fn close_door(&mut self) {
        let duty = duty_from_angle(0, self.min_duty, self.duty_gap);
        self.channel.set_duty_cycle(duty).unwrap();
        Timer::after_millis(20).await;
    }
}

const fn duty_from_angle(deg: u32, min_duty: u32, duty_gap: u32) -> u16 {
    let duty = min_duty + ((deg * duty_gap) / 180);
    duty as u16
}
