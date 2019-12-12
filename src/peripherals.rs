use core::cell::RefCell;
use cortex_m::{interrupt::Mutex, peripheral::syst::SystClkSource::Core};
use stm32f0xx_hal::delay::Delay;
use stm32f0xx_hal::{prelude::*, stm32, stm32::{Interrupt, Peripherals, EXTI},};

pub struct Shared {
    pub adc: stm32f0xx_hal::adc::Adc,
    pub led:
        stm32f0xx_hal::gpio::gpiob::PB1<stm32f0xx_hal::gpio::Output<stm32f0xx_hal::gpio::PushPull>>,
    pub tx: stm32f0xx_hal::serial::Tx<stm32::USART1>,
    pub delay: Delay,
    pub exti: EXTI,
}

pub fn init_peripherals() -> Result<Shared, &'static str> {
    if let (Some(p), Some(cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        cortex_m::interrupt::free(move |cs| {
            // Enable clock for SYSCFG
            let rcc = p.RCC;
            rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());

            let mut flash = p.FLASH;
            let mut rcc = rcc.configure().hsi48().sysclk(48.mhz()).freeze(&mut flash);

            let gpioa = p.GPIOA.split(&mut rcc);
            let gpiob = p.GPIOB.split(&mut rcc);

            let mut syst = cp.SYST;
            let syscfg = p.SYSCFG;
            let exti = p.EXTI;

            let mut led = gpiob.pb1.into_push_pull_output(cs);

            // Initialise ADC
            let adc = stm32f0xx_hal::adc::Adc::new(p.ADC, &mut rcc);

            // USART1 at PA9 (TX) and PA10(RX)
            let tx = gpioa.pa9.into_alternate_af1(cs);
            let rx = gpioa.pa10.into_alternate_af1(cs);

            // Initialiase UART
            let (mut tx, _) =
                stm32f0xx_hal::serial::Serial::usart1(p.USART1, (tx, rx), 115_200.bps(), &mut rcc)
                    .split();

            // Get delay provider
            let mut delay = Delay::new(syst, &rcc);

            // Configure PB8 as input (button)
            let _ = gpiob.pb8.into_pull_down_input(cs);

            // Enable external interrupt for PB8
            syscfg.exticr3.modify(|_, w| unsafe { w.exti8().pb8() });

            // Set interrupt request mask for line 8
            exti.imr.modify(|_, w| w.mr8().set_bit());

            exti.rtsr.modify(|_, w| w.tr8().set_bit());

            // Enable EXTI IRQ, set prio 1 and clear any pending IRQs
            let mut nvic = cp.NVIC;
            unsafe {
                nvic.set_priority(Interrupt::EXTI4_15, 1);
                cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI4_15);
            }
            cortex_m::peripheral::NVIC::unpend(Interrupt::EXTI4_15);

            Ok(Shared {
                adc,
                led,
                tx,
                delay,
                exti,
            })
        })
    } else {
        Err("can't take peripherals")
    }
}
