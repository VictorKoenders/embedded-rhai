#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate panic_halt;
extern crate alloc;

use cortex_m::asm;
use cortex_m_rt::entry;
use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use rhai::{Engine, RegisterFn};
use stm32f1xx_hal::{prelude::*, device, delay::Delay};
use embedded_hal::digital::v2::OutputPin;
use core::cell::UnsafeCell;
use alloc::rc::Rc;

// this is the allocator the application will use
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();
const HEAP_SIZE: usize = 1024; // in bytes

#[entry]
fn main() -> ! {
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }

    let p = device::Peripherals::take().unwrap();
    let cp = device::CorePeripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpioc = p.GPIOC.split(&mut rcc.apb2);

    let pin = Rc::new(UnsafeCell::new(gpioc.pc13.into_open_drain_output(&mut gpioc.crh)));

    let mut engine = Engine::new();
    {
        let pin = pin.clone();
        engine.register_fn("p13_on", move || {
            let pin = unsafe { &mut *pin.get() };
            pin.set_low().unwrap();
        });
    }
    engine.register_fn("p13_off", move || {
        let pin = unsafe { &mut *pin.get() };
        pin.set_high().unwrap();
    });

    let delay = UnsafeCell::new(Delay::new(cp.SYST, clocks));
    engine.register_fn("sleep_ms", move |ms: u16| {
        let delay = unsafe { &mut *delay.get() };
        delay.delay_ms(ms);
    });

    let _result = engine.eval::<()>(r#"
loop {
    p13_on();
    sleep_ms(1000);
    p13_off();
    sleep_ms(1000);
}
"#);

    asm::bkpt();
    loop {
        asm::nop();
        // your code goes here
    }
}


// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    asm::bkpt();

    loop {}
}

