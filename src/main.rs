//! atsamd-demo
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = atsamd51n, peripherals = true )]
mod app {
    use cortex_m_rtic_trace::{
        self, trace, GlobalTimestampOptions, LocalTimestampOptions, TimestampClkSrc,
        TraceConfiguration, TraceProtocol,
    };
    use atsamd51n::Interrupt;

    #[shared]
    struct SharedResources {}

    #[local]
    struct LocalResources {}

    #[init]
    fn init(mut ctx: init::Context) -> (SharedResources, LocalResources, init::Monotonics()) {
        rtic::pend(Interrupt::EIC_EXTINT_0);

        (SharedResources {}, LocalResources {}, init::Monotonics())
    }

    #[task(binds = EIC_EXTINT_0)]
    fn hardware(_: hardware::Context) {
        cortex_m::asm::bkpt();
    }
}
