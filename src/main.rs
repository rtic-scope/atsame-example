//! atsamd-demo
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = atsamd_hal::target_device, peripherals = true, dispatchers = [EIC_EXTINT_1])]
mod app {
    use atsamd_hal::target_device::Interrupt;
    use cortex_m_rtic_trace::{
        self, trace, GlobalTimestampOptions, LocalTimestampOptions, TimestampClkSrc,
        TraceConfiguration, TraceProtocol,
    };

    #[shared]
    struct SharedResources {}

    #[local]
    struct LocalResources {}

    #[init]
    fn init(mut ctx: init::Context) -> (SharedResources, LocalResources, init::Monotonics()) {
        cortex_m_rtic_trace::configure(
            &mut ctx.core.DCB,
            &mut ctx.core.TPIU,
            &mut ctx.core.DWT,
            &mut ctx.core.ITM,
            1, // task enter DWT comparator ID
            2, // task exit DWT comparator ID
            &TraceConfiguration {
                delta_timestamps: LocalTimestampOptions::Enabled,
                absolute_timestamps: GlobalTimestampOptions::Disabled,
                timestamp_clk_src: TimestampClkSrc::AsyncTPIU,
                tpiu_freq: 48_000_000, // Hz
                tpiu_baud: 115_200,    // B/s
                protocol: TraceProtocol::AsyncSWONRZ,
            },
        )
        .unwrap();

        rtic::pend(Interrupt::EIC_EXTINT_0);

        (SharedResources {}, LocalResources {}, init::Monotonics())
    }

    #[task(binds = EIC_EXTINT_0)]
    fn hardware(_: hardware::Context) {
        software::spawn().unwrap();
    }

    #[trace]
    #[task]
    fn software(_: software::Context) {

        #[trace]
        fn nested() {}

        // for _ in 0..10000 {
        //     nested();
        // }

        cortex_m::asm::bkpt();
    }
}
