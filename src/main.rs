#![no_std]
#![no_main]

mod fsm;
mod peripherals;

#[cfg(debug_assertions)]
use panic_semihosting as _;

#[cfg(not(debug_assertions))]
use panic_abort as _;

use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use heapless::mpmc::Q8;

static Q: Q8<fsm::Event> = Q8::new();

#[entry]
fn main() -> ! {
    let mut system_fsm = fsm::Machine::new();
    if system_fsm.event(fsm::Event::POST(fsm::Post{})).is_err() {
        loop {
            if let Some(x) = Q.dequeue() {
                if system_fsm.event(x).is_err() {
                    match system_fsm.state() {
                        fsm::State::BatteryVoltageLow(_) => {
                            // TODO: beep
                        },
                        fsm::State::Overcurrent(_) => {
                            // TODO: beep
                        },
                        _ => {}
                    }
                }
            } else {
                asm::wfi();
            }
        }
    } else {
        asm::bkpt();
        loop {}
    }
}
