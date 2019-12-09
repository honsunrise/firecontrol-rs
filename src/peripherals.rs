use stm32g4::stm32g431;
use core::result;
use cortex_m::peripheral::syst::SystClkSource;

pub fn init_peripherals() -> result::Result<(), &'static str> {
    if let Some(p) = cortex_m::Peripherals::take() {
        let mut syst = p.SYST;

        // configures the system timer to trigger a SysTick exception every second
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(12_000_000);
        syst.enable_counter();
        syst.enable_interrupt();
        Ok(())
    } else if let Some(p) = stm32g431::Peripherals::take() {
        let gpioc = &p.GPIOC;
        let rcc = &p.RCC;

        // enable the GPIO clock for IO port C
        rcc.ahb2enr.write(|w| w.gpiocen().set_bit());
        gpioc.moder.write(|w| unsafe { w.moder13().bits(0b11) });
        gpioc.otyper.write(|w| w.ot13().set_bit());
        gpioc.ospeedr.write(|w| unsafe { w.ospeedr13().bits(0b11) });
        gpioc.pupdr.write(|w| unsafe { w.pupdr13().bits(0b11) });
        Ok(())
    } else {
        Err("can't take cortex m peripherals")
    }
}

pub fn post() -> result::Result<(), &'static str> {
    Ok(())
}