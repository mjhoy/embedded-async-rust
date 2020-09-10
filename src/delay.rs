use crate::bool_future::*;
use f3::hal::stm32f30x::RCC;
use f3::hal::stm32f30x::{tim6, TIM6};
pub use f3::{hal::stm32f30x::rcc, led::Leds};

pub fn init_timer() -> &'static tim6::RegisterBlock {
    let rcc: &'static rcc::RegisterBlock = unsafe { &*RCC::ptr() };

    // Power on the TIM6 timer
    rcc.apb1enr.modify(|_, w| w.tim6en().set_bit());

    let tim6: &'static tim6::RegisterBlock = unsafe { &*TIM6::ptr() };

    // OPM Select one pulse mode
    // CEN Keep the counter disabled for now
    tim6.cr1.write(|w| w.opm().set_bit().cen().clear_bit());

    // Configure the prescaler to have the counter operate at 1 KHz
    // APB1_CLOCK = 8 MHz
    // PSC = 7999
    // 8 MHz / (7999 + 1) = 1 KHz
    // The counter (CNT) will increase on every millisecond
    tim6.psc.write(|w| w.psc().bits(7_999));

    tim6
}

pub async fn delay(ms: u16, tim6: &'static tim6::RegisterBlock) -> () {
    // set timer to go off in `ms` milliseconds
    tim6.arr.write(|w| w.arr().bits(ms));

    // CEN: enable the counter
    tim6.cr1.modify(|_, w| w.cen().set_bit());

    BoolFuture(|| tim6.sr.read().uif().bit_is_set()).await;

    // clear the update event flag
    tim6.sr.modify(|_, w| w.uif().clear_bit());
}
