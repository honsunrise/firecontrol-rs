use core::fmt::{self, Write};
use cortex_m::interrupt;

#[cfg(debug_assertions)]
use cortex_m_semihosting::hio;
#[cfg(debug_assertions)]
use cortex_m_semihosting::hio::HStdout;

#[macro_export]
macro_rules! println {
    () => {
        crate::print::out_str("\n")
    };
    ($s:expr) => {
        crate::print::out_str(concat!($s, "\n"))
    };
    ($s:expr, $($tt:tt)*) => {
        crate::print::out_fmt(format_args!(concat!($s, "\n"), $($tt)*))
    };
}

#[cfg(debug_assertions)]
static mut STDOUT: Option<HStdout> = None;

#[cfg(not(debug_assertions))]
static mut STDOUT: Option<NoStdout> = None;

#[cfg(debug_assertions)]
fn stdout() -> Result<HStdout, ()> {
    hio::hstdout()
}
#[cfg(not(debug_assertions))]
fn stdout() -> Result<NoStdout, ()> {
    Ok(NoStdout {})
}

pub fn out_str(s: &str) -> Result<(), ()> {
    interrupt::free(|_| unsafe {
        if STDOUT.is_none() {
            STDOUT = Some(stdout()?);
        }

        STDOUT.as_mut().unwrap().write_str(s).map_err(drop)
    })
}

pub fn out_fmt(args: fmt::Arguments) -> Result<(), ()> {
    interrupt::free(|_| unsafe {
        if STDOUT.is_none() {
            STDOUT = Some(stdout()?);
        }

        STDOUT.as_mut().unwrap().write_fmt(args).map_err(drop)
    })
}

pub struct NoStdout {}

impl fmt::Write for NoStdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Ok(())
    }
}
