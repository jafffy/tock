//! Implements the Tock bootloader.

use core::cell::Cell;
use core::cmp;
// use core::Result;
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use kernel::process::Error;

extern crate tockloader_proto;


pub static mut BUF: [u8; 512] = [0; 512];


const ESCAPE_CHAR: u8 = 0xFC;
const CMD_PING: u8 = 0x01;

const RES_PONG: u8 = 0x11;

pub struct Bootloader<'a, U: hil::uart::UARTAdvanced + 'a, F: hil::flash::Flash + 'static, G: hil::gpio::Pin + 'a> {
    uart: &'a U,
    flash: &'a F,
    select_pin: &'a G,
    led: &'a G,
    dpin: &'a G,
    /// Buffer correctly sized for the underlying flash page size.
    page_buffer: TakeCell<'static, F::Page>,
    // in_progress: Cell<Option<AppId>>,
    buffer: TakeCell<'static, [u8]>,
    // baud_rate: u32,
    // response: TakeCell<'a, tockloader_proto::Response<'a>>,
    pinged: Cell<bool>,
}

impl<'a, U: hil::uart::UARTAdvanced + 'a, F: hil::flash::Flash + 'a, G: hil::gpio::Pin + 'a> Bootloader<'a, U, F, G> {
    pub fn new(uart: &'a U, flash: &'a F, select_pin: &'a G, led: &'a G, dpin: &'a G,
               page_buffer: &'static mut F::Page,
               buffer: &'static mut [u8])
               -> Bootloader<'a, U, F, G> {
        Bootloader {
            uart: uart,
            flash: flash,
            select_pin: select_pin,
            led: led,
            dpin: dpin,
            // in_progress: Cell::new(None),
            page_buffer: TakeCell::new(page_buffer),
            buffer: TakeCell::new(buffer),
            // response: TakeCell::empty(),
            pinged: Cell::new(false),
        }
    }

    pub fn initialize(&self) {

        // Setup UART and start listening.
        self.uart.init(hil::uart::UARTParams {
            baud_rate: 115200,
            stop_bits: hil::uart::StopBits::One,
            parity: hil::uart::Parity::None,
            hw_flow_control: false,
        });



        // // self.select_pin.enable();
        // self.select_pin.make_input();



        // // Check the select pin to see if we should enter bootloader mode.
        // let mut samples = 10000;
        // let mut active = 0;
        // let mut inactive = 0;
        // while samples > 0 {
        //     if self.select_pin.read() == false {
        //         active += 1;
        //     } else {
        //         inactive += 1;
        //     }
        //     samples -= 1;
        // }

        // if active > inactive {
            // Looks like we do want bootloader mode.





            self.buffer.take().map(|buffer| {
                self.dpin.toggle();
                self.led.toggle();
                self.uart.receive_automatic(buffer, 250);
                // self.uart.receive(buffer, 5);
                // buffer[0] = 97;
                // buffer[1] = 98;
                // buffer[2] = 100;
                // buffer[3] = 105;
                // buffer[4] = 110;
                // self.uart.transmit(buffer, 5);
            });




        // } else {
        //     // Jump to the kernel and start the real code.
        // }


    }

    // /// Internal helper function for setting up a new send transaction
    // fn send_new(&self, app_id: AppId, app: &mut App, len: usize) -> ReturnCode {
    //     match app.write_buffer.take() {
    //         Some(slice) => {
    //             app.write_len = cmp::min(len, slice.len());
    //             app.write_remaining = app.write_len;
    //             self.send(app_id, app, slice);
    //             ReturnCode::SUCCESS
    //         }
    //         None => ReturnCode::EBUSY,
    //     }
    // }

    // /// Internal helper function for continuing a previously set up transaction
    // /// Returns true if this send is still active, or false if it has completed
    // fn send_continue(&self, app_id: AppId, app: &mut App) -> Result<bool, ReturnCode> {
    //     if app.write_remaining > 0 {
    //         app.write_buffer.take().map_or(Err(ReturnCode::ERESERVE), |slice| {
    //             self.send(app_id, app, slice);
    //             Ok(true)
    //         })
    //     } else {
    //         Ok(false)
    //     }
    // }

    // /// Internal helper function for sending data for an existing transaction.
    // /// Cannot fail. If can't send now, it will schedule for sending later.
    // fn send(&self, app_id: AppId, app: &mut App, slice: AppSlice<Shared, u8>) {
    //     if self.in_progress.get().is_none() {
    //         self.in_progress.set(Some(app_id));
    //         self.tx_buffer.take().map(|buffer| {
    //             let mut transaction_len = app.write_remaining;
    //             for (i, c) in slice.as_ref()[slice.len() - app.write_remaining..slice.len()]
    //                 .iter()
    //                 .enumerate() {
    //                 if buffer.len() <= i {
    //                     break;
    //                 }
    //                 buffer[i] = *c;
    //             }

    //             // Check if everything we wanted to print
    //             // fit in the buffer.
    //             if app.write_remaining > buffer.len() {
    //                 transaction_len = buffer.len();
    //                 app.write_remaining -= buffer.len();
    //                 app.write_buffer = Some(slice);
    //             } else {
    //                 app.write_remaining = 0;
    //             }

    //             self.uart.transmit(buffer, transaction_len);
    //         });
    //     } else {
    //         app.pending_write = true;
    //         app.write_buffer = Some(slice);
    //     }
    // }

    fn send_response (&self, response: tockloader_proto::Response<'a>) {
        // self.response.map(|response| {

            self.buffer.take().map(|buffer| {
                let mut encoder = tockloader_proto::ResponseEncoder::new(&response).unwrap();
                let mut i = 0;
                while let Some(byte) = encoder.next() {
                    // uart.putc(byte);
                    buffer[i] = byte;
                    i += 1;
                }

                self.uart.transmit(buffer, i);
            });
        // });
    }
}

impl<'a, U: hil::uart::UARTAdvanced + 'a, F: hil::flash::Flash + 'a, G: hil::gpio::Pin + 'a> hil::uart::Client for Bootloader<'a, U, F, G> {
    fn transmit_complete(&self, buffer: &'static mut [u8], _error: hil::uart::Error) {
// self.led.clear();
        // self.buffer.replace(buffer);
        // self.uart.receive_automatic(buffer, 250);
        self.uart.receive(buffer, 3);

    }

    fn receive_complete(&self,
                        buffer: &'static mut [u8],
                        rx_len: usize,
                        _error: hil::uart::Error) {


        if self.pinged.get() == true {
            // self.led.clear();
        }


        let mut decoder = tockloader_proto::CommandDecoder::new();

        // decoder.read(buffers.slice(rx_len), )


        // loop {
        // if let Ok(Some(ch)) = uart.getc_try() {
        // let mut response = None;
        // let mut command: Result<Option<tockloader_proto::Command<'_>>, tockloader_proto::Error>;
        let mut need_reset = false;
        for i in 0..rx_len {

            // response = match decoder.receive(buffer[i]) {
            match decoder.receive(buffer[i]) {
                Ok(None) => {},
                Ok(Some(tockloader_proto::Command::Ping)) => {

                    self.buffer.replace(buffer);
                    self.send_response(tockloader_proto::Response::Pong);
                    break;
                }
                Ok(Some(tockloader_proto::Command::Reset)) => {
                    need_reset = true;
                    // decoder.reset();
    // self.led.clear();
                    // self.uart.receive_automatic(buffer, 250);
                    self.uart.receive(buffer, 3);
                    break;
                },
                Ok(Some(tockloader_proto::Command::GetAttr{index})) => {
                    // Some(tockloader_proto::Response::Unknown)
    self.led.clear();
                    break;
                }
                Ok(Some(_)) => {
                    self.send_response(tockloader_proto::Response::Unknown);
                    // Some(tockloader_proto::Response::Unknown)
    // self.led.clear();
                    break;
                }
                Err(_) => {
                    self.send_response(tockloader_proto::Response::InternalError);

                    // Some(tockloader_proto::Response::InternalError)
                    break;
                }
            };


        }
        if need_reset {
            // self.led.clear();
            self.pinged.set(true);
            decoder.reset();

        }

        // let response = match command {
        //     Ok(None) => None,
        //     Ok(Some(tockloader_proto::Command::Ping)) => Some(tockloader_proto::Response::Pong),
        //     Ok(Some(tockloader_proto::Command::Reset)) => {
        //         // need_reset = true;
        //         None
        //     },
        //     Ok(Some(tockloader_proto::Command::GetAttr{index})) => Some(tockloader_proto::Response::Unknown),
        //     Ok(Some(_)) => Some(tockloader_proto::Response::Unknown),
        //     Err(_) => Some(tockloader_proto::Response::InternalError),
        // };

//         if let Some(response) = response {
// self.led.toggle();
//             let mut encoder = tockloader_proto::ResponseEncoder::new(&response).unwrap();
//             let mut i = 0;
//             while let Some(byte) = encoder.next() {
//                 // uart.putc(byte);
//                 buffer[i] = byte;
//                 i += 1;
//             }

//             self.uart.transmit(buffer, i);
//         }

        // }





// self.dpin.toggle();

//         // Check for escape character then the command byte.
//         if rx_len >= 2 && buffer[rx_len-2] == ESCAPE_CHAR && buffer[rx_len-1] != ESCAPE_CHAR {
//             // This looks like a valid command.

//             match buffer[rx_len-1] {
//                 CMD_PING => {
//                     buffer[0] = ESCAPE_CHAR;
//                     buffer[1] = RES_PONG;

//                     self.uart.transmit(buffer, 2);
//                 }

//                 _ => {
//     self.led.clear();
//                     self.page_buffer.take().map(move |page| {
//                         for i in 0..rx_len {
//                             page.as_mut()[i] = buffer[i];
//                         }
//         // self.led.clear();
//                         // self.buffer.replace(buffer);
//                         self.uart.receive_automatic(buffer, 250);
//                         self.flash.write_page(384, page);
//                     });
//                 }
//             }

//         }
    }
}

impl<'a, U: hil::uart::UARTAdvanced + 'a, F: hil::flash::Flash + 'a, G: hil::gpio::Pin + 'a> hil::flash::Client<F> for Bootloader<'a, U, F, G> {
    fn read_complete(&self, pagebuffer: &'static mut F::Page, _error: hil::flash::Error) {

    }

    fn write_complete(&self, pagebuffer: &'static mut F::Page, _error: hil::flash::Error) {
// self.led.toggle();
        self.page_buffer.replace(pagebuffer);
    }

    fn erase_complete(&self, _error: hil::flash::Error) {}
}
