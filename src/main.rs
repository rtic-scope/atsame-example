//! atsamd-demo
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = atsamd_hal::target_device, peripherals = true, dispatchers = [EIC_EXTINT_1])]
mod app {
    use atsamd_hal::target_device::Interrupt;
    use atsamd_hal::thumbv7em::clock::GenericClockController;
    use cortex_m_rtic_trace::{
        self, trace, GlobalTimestampOptions, LocalTimestampOptions, TimestampClkSrc,
        TraceConfiguration, TraceProtocol,
    };
    use atsamd_hal as hal;
    use hal::gpio::v2::pin::{Alternate, M};

    #[shared]
    struct SharedResources {}

    #[local]
    struct LocalResources {}

    #[init]
    fn init(mut ctx: init::Context) -> (SharedResources, LocalResources, init::Monotonics()) {
        // configure trace clock
        let mut gcc = GenericClockController::with_internal_32kosc(
            ctx.device.GCLK,
            &mut ctx.device.MCLK,
            &mut ctx.device.OSC32KCTRL,
            &mut ctx.device.OSCCTRL,
            &mut ctx.device.NVMCTRL,
        );
        let gclk0 = gcc.gclk0();
        let trace_clk = gcc.cm4_trace(&gclk0).unwrap();

        let freq = trace_clk.freq().0;

        // configure SWO pin
        let pins = hal::gpio::v2::Pins::new(ctx.device.PORT);
        let _pc27 = pins.pc27.into_mode::<Alternate<M>>();

        // configure tracing
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
                tpiu_freq: freq,    // Hz
                tpiu_baud: 38400, // B/s
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
        // #[trace]
        // fn nested() {}

        // // for _ in 0..10000 {
        // //     nested();
        // // }

        // cortex_m::asm::bkpt();
    }
}
