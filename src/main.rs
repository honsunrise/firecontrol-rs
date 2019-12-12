#![no_std]
#![no_main]
#![feature(const_fn, const_raw_ptr_deref, maybe_uninit_ref)]

mod fsm;
mod peripherals;
mod print;
mod queue;

#[macro_use]
mod utils;

#[cfg(debug_assertions)]
use panic_semihosting as _;

#[cfg(not(debug_assertions))]
use panic_abort as _;

use crate::peripherals::Shared;
use crate::queue::AtomicQueue;
use core::cell::{Cell, RefCell};
use core::hint::unreachable_unchecked;
use core::mem::MaybeUninit;
#[doc(hidden)]
pub use core::ops::Deref as __Deref;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use cortex_m::asm;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::{entry, exception};
use stm32f0xx_hal::{
    prelude::*,
    stm32,
    stm32::{interrupt, Interrupt, Peripherals, EXTI},
};

pub struct Lazy<T: Sync>(Cell<MaybeUninit<T>>, AtomicBool);

impl<T: Sync> Lazy<T> {
    pub const INIT: Self = Lazy(Cell::new(MaybeUninit::uninit()), AtomicBool::new(false));

    #[inline(always)]
    pub fn get<F>(&'static self, f: F) -> &T
        where
            F: FnOnce() -> T,
    {
        if self.1.load(Ordering::Acquire) == false {
            unsafe {
                (*self.0.as_ptr()).as_mut_ptr().write(f());
            }
            self.1.store(true, Ordering::Release)
        }

        unsafe { &*(*self.0.as_ptr()).as_ptr() as &T }
    }
}

unsafe impl<T: Sync> Sync for Lazy<T> {}

static QUEUE: QUEUE = QUEUE {
    __private_field: (),
};

#[allow(missing_copy_implementations)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
struct QUEUE {
    __private_field: (),
}

impl __Deref for QUEUE {
    type Target = Mutex<RefCell<AtomicQueue<'static, fsm::Event>>>;

    fn deref(&self) -> &Mutex<RefCell<AtomicQueue<'static, fsm::Event>>> {
        #[inline(always)]
        fn __static_ref_initialize() -> Mutex<RefCell<AtomicQueue<'static, fsm::Event>>> {
            Mutex::new(RefCell::new(AtomicQueue::new()))
        }

        #[inline(always)]
        fn __stability() -> &'static Mutex<RefCell<AtomicQueue<'static, fsm::Event>>> {
            static LAZY: Lazy<Mutex<RefCell<AtomicQueue<'static, fsm::Event>>>> = Lazy::INIT;
            LAZY.get(__static_ref_initialize)
        }
        __stability()
    }
}

static BOARD: BOARD = BOARD {
    __private_field: (),
};

#[allow(missing_copy_implementations)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
struct BOARD {
    __private_field: (),
}

impl __Deref for BOARD {
    type Target = Mutex<RefCell<Shared>>;

    fn deref(&self) -> &Mutex<RefCell<Shared>> {
        #[inline(always)]
        fn __static_ref_initialize() -> Mutex<RefCell<Shared>> {
            Mutex::new(RefCell::new(peripherals::init_peripherals().unwrap()))
        }

        #[inline(always)]
        fn __stability() -> &'static Mutex<RefCell<Shared>> {
            static LAZY: Lazy<Mutex<RefCell<Shared>>> = Lazy::INIT;
            LAZY.get(__static_ref_initialize)
        }
        __stability()
    }
}

#[entry]
fn main() -> ! {
    let mut system_fsm = fsm::Machine::new();
    if system_fsm.event(fsm::Event::POST(fsm::Post {})).is_ok() {
        loop {
            // Enter critical section
            cortex_m::interrupt::free(|cs| {
                let mut board = BOARD.borrow(cs).borrow_mut();
                let queue = QUEUE.borrow(cs).borrow();

                if let Some(event) = queue.pop(cs) {
                    // Read temperature data from internal sensor using ADC
                    let t = stm32f0xx_hal::adc::VTemp::read(&mut board.adc, None);
                    println!("Temperature {}.{}C\r", t / 100, t % 100).ok();

                    // Read volatage reference data from internal sensor using ADC
                    let t = stm32f0xx_hal::adc::VRef::read_vdda(&mut board.adc);
                    println!("Vdda {}mV\r", t).ok();
                    board.led.toggle().ok();
                } else {
                    asm::wfi();
                }
            });
        }
    } else {
        asm::bkpt();
        loop {}
    }
}

#[interrupt]
fn EXTI4_15() {
    cortex_m::interrupt::free(|cs| {
        let mut board = BOARD.borrow(cs).borrow_mut();
        let queue = QUEUE.borrow(cs).borrow();

        queue.push(
            fsm::Event::BatteryVoltageChange(fsm::BatteryVoltageChange {}),
            cs,
        );

        board.exti.pr.write(|w| w.pif8().set_bit());
    })
}
