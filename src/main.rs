#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = atsamd_hal::target_device, peripherals = true, dispatchers = [EIC_EXTINT_1])]
mod app {
    use atsamd_hal::{
        self as hal, delay,
        ehal::prelude::*,
        gpio::v2::pin::{Alternate, M},
        target_device::Interrupt,
        thumbv7em::clock,
    };
    use cortex_m_rtic_trace::{
        self, trace, GlobalTimestampOptions, LocalTimestampOptions, TimestampClkSrc,
        TraceConfiguration, TraceProtocol,
    };
    use cortex_m;

    #[shared]
    struct SharedResources {}

    #[local]
    struct LocalResources {
        delay: delay::Delay,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (SharedResources, LocalResources, init::Monotonics()) {
        // configure trace clock
        let mut gcc = clock::GenericClockController::with_internal_32kosc(
            ctx.device.GCLK,
            &mut ctx.device.MCLK,
            &mut ctx.device.OSC32KCTRL,
            &mut ctx.device.OSCCTRL,
            &mut ctx.device.NVMCTRL,
        );
        let gclk0 = gcc.gclk0();
        let trace_clk = gcc.cm4_trace(&gclk0).unwrap();

        // Create a Delay for fake work
        let delay = delay::Delay::new(ctx.core.SYST, &mut gcc);

        // configure SWO pin
        let pins = hal::gpio::v2::Pins::new(ctx.device.PORT);
        let _pc27 = pins.pc27.into_mode::<Alternate<M>>();

        cortex_m::asm::bkpt(); // interactive escape from SWO pin transient

        // configure tracing
        cortex_m_rtic_trace::configure(
            &mut ctx.core.DCB,
            &mut ctx.core.TPIU,
            &mut ctx.core.DWT,
            &mut ctx.core.ITM,
            1, // task enter DWT comparator ID
            2, // task exit DWT comparator ID
            &TraceConfiguration {
                delta_timestamps: LocalTimestampOptions::Enabled, // enabled with a bypassed (= 1) prescaler
                absolute_timestamps: GlobalTimestampOptions::Disabled, // disable absolute timestamps
                timestamp_clk_src: TimestampClkSrc::AsyncTPIU,
                tpiu_freq: trace_clk.freq().0, // Hz
                tpiu_baud: 1_000_000,          // B/s
                protocol: TraceProtocol::AsyncSWONRZ,
            },
        )
        .unwrap();

        rtic::pend(Interrupt::EIC_EXTINT_0);

        (
            SharedResources {},
            LocalResources { delay },
            init::Monotonics(),
        )
    }

    #[task(binds = EIC_EXTINT_0, priority = 1)]
    fn hardware(_: hardware::Context) {
        software::spawn().unwrap();
    }

    #[trace]
    #[task(priority = 2, local = [delay])]
    fn software(ctx: software::Context) {
        ctx.local.delay.delay_ms(100_u8); // fake work
    }
}
