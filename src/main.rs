//! atsamd-demo
#![no_main]
#![no_std]

macro_rules! clear_screen {
    ($tx:tt) => {{
        // Sequence:
        // `<ESC>[2J` (clear screen)
        // `<ESC>[H` (cursor to home position)
        // `<ESC>` == 0x1b
        $tx.write_str("\x1b").unwrap();
        $tx.write_str("[2J").unwrap();
        $tx.write_str("\x1b").unwrap();
        $tx.write_str("[H").unwrap();
    }};
}

macro_rules! clear_line {
    ($tx:tt) => {{
        // Sequence:
        // `<ESC>[2K` (clear line)
        // `\r`: CR
        // `<ESC>` == 0x1b
        $tx.write_str("\x1b").unwrap();
        $tx.write_str("[2K").unwrap();
        $tx.write_str("\r").unwrap();
    }};
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        if let Some(u) = UART0_TX.as_mut() {
            write!(u as &mut dyn Write<_, Error = _>, "{}\r\n", info).unwrap()
        }
    }
    loop {
        cortex_m::asm::nop();
    }
}

use core::fmt::Write as _;

// Software Aes192
//use aes::Aes192;

use atsamd_hal::{
    aes::*,
    clock::GenericClockController,
    dsu::Dsu,
    gpio::v2::{Alternate, Pin, Pins, D, PA04, PA05},
    hal::serial::Write,
    nvm::{smart_eeprom::SmartEepromMode, Nvm},
    prelude::*,
    sercom::v2::{uart::*, IoSet3, Sercom0},
    time::U32Ext,
};

use nb::block;

use rtic::app;
pub type Uart0Tx =
    Uart<Config<Pads<Sercom0, IoSet3, Pin<PA05, Alternate<D>>, Pin<PA04, Alternate<D>>>>, TxDuplex>;
pub type Uart0Rx =
    Uart<Config<Pads<Sercom0, IoSet3, Pin<PA05, Alternate<D>>, Pin<PA04, Alternate<D>>>>, RxDuplex>;

pub type String = heapless::String<256>;

static mut UART0_TX: Option<Uart0Tx> = None;

#[app(device = atsamd_hal::target_device, peripherals = true, dispatchers = [FREQM])]
mod app {
    use super::*;

    #[shared]
    struct SharedResources {
        uart0_rx: Uart0Rx,
        nvm: Nvm,
        dsu: Dsu,
        buffer: String,
    }

    #[local]
    struct LocalResources {}

    #[init]
    fn init(cx: init::Context) -> (SharedResources, LocalResources, init::Monotonics()) {
        let mut device = cx.device;

        let pins = Pins::new(device.PORT);

        let mclk = &mut device.MCLK;

        let mut clocks = GenericClockController::with_external_32kosc(
            device.GCLK,
            mclk,
            &mut device.OSC32KCTRL,
            &mut device.OSCCTRL,
            &mut device.NVMCTRL,
        );

        let gclk0 = clocks.gclk0();

        let mut uart0 = Config::new(
            mclk,
            device.SERCOM0,
            Pads::default().rx(pins.pa05).tx(pins.pa04),
            clocks.sercom0_core(&gclk0).unwrap().freq(),
        )
        .baud(115_200.hz(), BaudMode::Arithmetic(Oversampling::Bits16))
        .enable();
        uart0.enable_interrupts(Flags::RXC);

        let nvm = Nvm::new(device.NVMCTRL);
        let dsu = Dsu::new(device.DSU, &device.PAC).unwrap();

        // Enable bus clocking for AES peripheral
        mclk.apbcmask.modify(|_, w| w.aes_().set_bit());

        let key = [
            0x54, 0x68, 0x61, 0x74, 0x73, 0x20, 0x6D, 0x79, 0x20, 0x4B, 0x75, 0x6E, 0x67, 0x20,
            0x46, 0x75,
        ];

        let message = [
            0x54, 0x77, 0x6F, 0x20, 0x4F, 0x6E, 0x65, 0x20, 0x4E, 0x69, 0x6E, 0x65, 0x20, 0x54,
            0x77, 0x6F,
        ];

        let ciphertext = *aes::Block::from_slice(&[
            0x29, 0xC3, 0x50, 0x5F, 0x57, 0x14, 0x20, 0xF6, 0x40, 0x22, 0x99, 0xB3, 0x1A, 0x02,
            0xD7, 0x3A,
        ]);

        let aeskey = GenericArray::from_slice(&key);
        let mut aesmsg = *aes::Block::from_slice(&message);

        // Atsamd peripheral init
        let aes = Aes::new(device.AES);
        let aes_reg = aes.destroy();

        // Store AES hardware peripheral in AesRustCrypto to ensure
        // no uses of the AES peripheral outside RustCrypto
        let aes_rust_crypto = Aes::new_rustcrypto_backend(aes_reg);

        let aes128 = aes_rust_crypto.new_128bit(aeskey);
        // Direct access to AES peripheral through RustCrypto
        //let aes128 = Aes128::new(aeskey);

        write!(
            &mut uart0 as &mut dyn Write<_, Error = _>,
            "Begin AES demo\r\n"
        )
        .unwrap();

        for x in aesmsg {
            write!(&mut uart0 as &mut dyn Write<_, Error = _>, "{}", x as char).unwrap();
        }
        write!(&mut uart0 as &mut dyn Write<_, Error = _>, "\r\n").unwrap();
        for x in aesmsg {
            write!(&mut uart0 as &mut dyn Write<_, Error = _>, "{:#04x} ", x).unwrap();
        }
        write!(&mut uart0 as &mut dyn Write<_, Error = _>, "\r\n").unwrap();

        // Encrypt message
        aes128.encrypt_block(&mut aesmsg);

        for x in aesmsg {
            write!(&mut uart0 as &mut dyn Write<_, Error = _>, "{}", x as char).unwrap();
        }
        write!(&mut uart0 as &mut dyn Write<_, Error = _>, "\r\n").unwrap();
        for x in aesmsg {
            write!(&mut uart0 as &mut dyn Write<_, Error = _>, "{:#04x} ", x).unwrap();
        }
        write!(
            &mut uart0 as &mut dyn Write<_, Error = _>,
            "   <- AES ciphertext\r\n"
        )
        .unwrap();
        for x in ciphertext {
            write!(&mut uart0 as &mut dyn Write<_, Error = _>, "{:#04x} ", x).unwrap();
        }
        write!(
            &mut uart0 as &mut dyn Write<_, Error = _>,
            "   <- known ciphertext\r\n"
        )
        .unwrap();

        assert_eq!(aesmsg, ciphertext);

        // Decrypt message
        aes128.decrypt_block(&mut aesmsg);

        for x in aesmsg {
            write!(&mut uart0 as &mut dyn Write<_, Error = _>, "{}", x as char).unwrap();
        }
        write!(&mut uart0 as &mut dyn Write<_, Error = _>, "\r\n").unwrap();
        for x in aesmsg {
            write!(&mut uart0 as &mut dyn Write<_, Error = _>, "{:#04x} ", x).unwrap();
        }
        write!(&mut uart0 as &mut dyn Write<_, Error = _>, "\r\n").unwrap();

        cortex_m::asm::bkpt();

        // AES RustCrypto Example

        let key = GenericArray::from_slice(&[0u8; 16]);
        let mut block = aes::Block::default();

        // Initialize cipher
        let cipher = atsamd_hal::aes::Aes128::new(key);

        let block_copy = block;

        // Encrypt block in-place
        cipher.encrypt_block(&mut block);

        // And decrypt it back
        cipher.decrypt_block(&mut block);
        assert_eq!(block, block_copy);

        cortex_m::asm::bkpt();

        // AES CMAC RustCrypto Example

        use cmac::{Cmac, Mac, NewMac};

        // Create `Mac` trait implementation, namely CMAC-AES128
        let mut mac = Cmac::<Aes128>::new_from_slice(b"very secret key.").unwrap();
        mac.update(b"input message");

        // `result` has type `Output` which is a thin wrapper around array of
        // bytes for providing constant time equality check
        let result = mac.finalize();
        // To get underlying array use the `into_bytes` method, but be careful,
        // since incorrect use of the tag value may permit timing attacks which
        // defeat the security provided by the `Output` wrapper
        let tag_bytes = result.into_bytes();

        //To verify the message:

        let mut mac = Cmac::<Aes128>::new_from_slice(b"very secret key.").unwrap();

        mac.update(b"input message");

        // `verify` will return `Ok(())` if tag is correct, `Err(MacError)` otherwise
        mac.verify(&tag_bytes).unwrap();
        cortex_m::asm::bkpt();

        // AES Counter RustCrypto Example
        use ctr::cipher::{NewCipher, StreamCipher, StreamCipherSeek};

        // `aes` crate provides AES block cipher implementation
        type Aes128Ctr = ctr::Ctr128BE<atsamd_hal::aes::Aes128>;

        let mut data = [1, 2, 3, 4, 5, 6, 7];

        let key = b"very secret key.";
        let nonce = b"and secret nonce";

        // create cipher instance
        let mut cipher = Aes128Ctr::new(key.into(), nonce.into());

        // apply keystream (encrypt)
        cipher.apply_keystream(&mut data);
        assert_eq!(data, [6, 245, 126, 124, 180, 146, 37]);

        // seek to the keystream beginning and apply it again to the `data` (decrypt)
        cipher.seek(0);
        cipher.apply_keystream(&mut data);
        assert_eq!(data, [1, 2, 3, 4, 5, 6, 7]);

        cortex_m::asm::bkpt();

        write!(
            &mut uart0 as &mut dyn Write<_, Error = _>,
            "RTIC booted!\r\n"
        )
        .unwrap();

        let (uart0_rx, uart0_tx) = uart0.split();

        unsafe {
            UART0_TX.replace(uart0_tx);
        }

        (
            SharedResources {
                uart0_rx,
                nvm,
                dsu,
                buffer: heapless::String::new(),
            },
            LocalResources {},
            init::Monotonics(),
        )
    }

    #[derive(Debug)]
    pub enum Action {
        Read,
        Write,
    }

    #[task(shared = [buffer, nvm], capacity = 10)]
    fn uart_handle(cx: uart_handle::Context, uart_data: UartCommand) {
        let mut buffer = cx.shared.buffer;
        let mut nvm = cx.shared.nvm;
        let uart0_tx = unsafe { UART0_TX.as_mut().unwrap() as &mut dyn Write<_, Error = _> };
        match uart_data {
            UartCommand::Return => {
                buffer.lock(|b| {
                    uart0_tx.write_str("\r\n").unwrap();
                    // custom action start

                    let mut iterator = b.split_whitespace();
                    let (action, arg1, arg2) = match iterator
                        .next()
                        .and_then(|v| match v {
                            "r" => Some(Action::Read),
                            "w" => Some(Action::Write),
                            _ => None,
                        })
                        .and_then(|action| {
                            iterator
                                .next()
                                .and_then(|arg1| arg1.parse().ok())
                                .and_then(|arg1| {
                                    iterator
                                        .next()
                                        .and_then(|arg2| arg2.parse().ok())
                                        .map(|arg2| (action, arg1, arg2))
                                })
                        }) {
                        Some(v) => v,
                        None => {
                            uart0_tx.write_str("argument parsing failure\r\n").unwrap();
                            b.clear();
                            return;
                        }
                    };

                    nvm.lock(|n| {
                        let mut se = match n.smart_eeprom().unwrap() {
                            SmartEepromMode::Unlocked(se) => se,
                            SmartEepromMode::Locked(se) => se.unlock(),
                        };
                        match action {
                            Action::Read => {
                                se.iter::<u8>().enumerate().skip(arg1).take(arg2).for_each(
                                    |(i, v)| {
                                        write!(
                                            uart0_tx as &mut dyn Write<_, Error = _>,
                                            "{:0x}: {:0x}\r\n",
                                            i, v
                                        )
                                        .unwrap()
                                    },
                                )
                            }
                            Action::Write => {
                                use core::convert::TryInto;
                                uart0_tx.write_str("writing to eeprom..\r\n").unwrap();
                                se.iter_mut::<u8>()
                                    .skip(arg1)
                                    .take(arg2)
                                    .for_each(|v| *v = (arg2 & 0xff_usize).try_into().unwrap());
                                uart0_tx.write_str("done\r\n").unwrap();
                            }
                        }
                    });

                    b.clear();
                })
            }
            UartCommand::DataChar(character) => buffer.lock(|b| match b.push(character) {
                Ok(_) => {
                    clear_line!(uart0_tx);
                    uart0_tx.write_str(b.as_str()).unwrap();
                }
                Err(_) => uart_handle::spawn(UartCommand::BufferFull).unwrap(),
            }),
            UartCommand::Backspace => buffer.lock(|b| {
                if b.pop().is_some() {
                    clear_line!(uart0_tx);
                    uart0_tx.write_str(b.as_str()).unwrap();
                }
            }),
            UartCommand::CtrlC => buffer.lock(|b| {
                b.clear();
                clear_screen!(uart0_tx);
                uart0_tx.write_str(b.as_str()).unwrap();
            }),
            // failure / not supported commands
            other => write!(
                uart0_tx as &mut dyn Write<_, Error = _>,
                "error: {:?}\r\n",
                other
            )
            .unwrap(),
        }
    }

    #[task(binds = SERCOM0_2, shared = [uart0_rx], priority = 2)]
    fn uart_interrupt(cx: uart_interrupt::Context) {
        let mut rx = cx.shared.uart0_rx;
        match rx.lock(|rx| block!(rx.read())) {
            Ok(byte) => match byte as char {
                '\u{7f}' => uart_handle::spawn(UartCommand::Backspace),
                '\u{3}' => uart_handle::spawn(UartCommand::CtrlC),
                '\r' => uart_handle::spawn(UartCommand::Return),
                byte => uart_handle::spawn(UartCommand::DataChar(byte)),
            },
            Err(e) => uart_handle::spawn(UartCommand::ReadError(e)),
        }
        .unwrap();
    }
}

#[derive(Debug)]
pub enum UartCommand {
    CtrlC,
    Backspace,
    DataChar(char),
    Return,
    BufferFull,
    ReadError(Error),
}
