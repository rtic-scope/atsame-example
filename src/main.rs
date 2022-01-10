//! atsamd-demo
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = atsamd51n, peripherals = true )]
mod app {
    use atsamd51n::Interrupt;
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
        rtic::pend(Interrupt::EIC_EXTINT_0);

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
                tpiu_freq: 16_000_000, // Hz
                tpiu_baud: 115_200,    // B/s
                protocol: TraceProtocol::AsyncSWONRZ,
            },
        )
        .unwrap();

        (SharedResources {}, LocalResources {}, init::Monotonics())
    }

    #[task(binds = EIC_EXTINT_0)]
    fn hardware(_: hardware::Context) {
        cortex_m::asm::bkpt();
    }
}
