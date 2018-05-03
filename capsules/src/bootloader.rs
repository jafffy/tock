//! Implements the Tock bootloader.

use core::cell::Cell;
use core::cmp;
use kernel::common::take_cell::TakeCell;
use kernel::hil;

extern crate tockloader_proto;


pub static mut BUF: [u8; 512] = [0; 512];
// pub static mut BUF2: [u8; 128] = [0; 128];
// pub static mut BUF3: [u8; 128] = [0; 128];


const ESCAPE_CHAR: u8 = 0xFC;

const RES_OK: u8 = 0x15;
const RES_READ_RANGE: u8 = 0x20;
const RES_GET_ATTR: u8 = 0x22;

#[derive(Copy, Clone, PartialEq)]
enum state {
    Idle,
    ErasePage,
    GetAttribute{index: u8},
    SetAttribute{index: u8},
    ReadRange{address: u32, length: u16, remaining_length: u16},
}

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
    // buffer2: TakeCell<'static, [u8]>,
    // buffer3: TakeCell<'static, [u8]>,
    // baud_rate: u32,
    // response: TakeCell<'a, tockloader_proto::Response<'a>>,
    pinged: Cell<bool>,
    state: Cell<state>,
}

impl<'a, U: hil::uart::UARTAdvanced + 'a, F: hil::flash::Flash + 'a, G: hil::gpio::Pin + 'a> Bootloader<'a, U, F, G> {
    pub fn new(uart: &'a U, flash: &'a F, select_pin: &'a G, led: &'a G, dpin: &'a G,
               page_buffer: &'static mut F::Page,
               buffer: &'static mut [u8])
               // buffer2: &'static mut [u8],
               // buffer3: &'static mut [u8])
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
            // buffer2: TakeCell::new(buffer2),
            // buffer3: TakeCell::new(buffer3),
            // response: TakeCell::empty(),
            pinged: Cell::new(false),
            state: Cell::new(state::Idle),
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
                // self.uart.receive(buffer, 2);
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
    fn transmit_complete(&self, buffer: &'static mut [u8], error: hil::uart::Error) {
        if error != hil::uart::Error::CommandComplete {
            // self.led.clear();
        } else {

            match self.state.get() {

                // Check if there is more to be read, and if so, read it and
                // send it.
                state::ReadRange{address, length: _, remaining_length} => {
                    // We have sent some of the read range to the client.
                    // We are either done, or need to setup the next read.
                    if remaining_length == 0 {
                        self.state.set(state::Idle);
                        self.uart.receive_automatic(buffer, 250);

                    } else {
                        self.buffer.replace(buffer);
                        self.page_buffer.take().map(move |page| {
                            let page_size = page.as_mut().len();
                            self.flash.read_page(address as usize / page_size, page);
                        });
                    }
                }

                _ => {
                    self.uart.receive_automatic(buffer, 250);
                }
            }
        }

    }

    fn receive_complete(&self,
                        buffer: &'static mut [u8],
                        rx_len: usize,
                        error: hil::uart::Error) {


        if error != hil::uart::Error::CommandComplete {
            // self.led.clear();
            return
        }


    //     if self.pinged.get() == true {
    //         self.led.clear();

    //             self.page_buffer.take().map(move |page| {
    //                 page.as_mut()[0] = 0xa1;
    //                 for i in 0..rx_len {
    //                     page.as_mut()[i+1] = buffer[i];
    //                 }
    // // self.led.clear();
    //                 // self.buffer.replace(buffer);
    //                 // self.uart.receive_automatic(buffer, 250);
    //                 self.flash.write_page(384, page);
    //             });
    //             return;
    //     }


        let mut decoder = tockloader_proto::CommandDecoder::new();

        // decoder.read(buffers.slice(rx_len), )


        // loop {
        // if let Ok(Some(ch)) = uart.getc_try() {
        // let mut response = None;
        // let mut command: Result<Option<tockloader_proto::Command<'_>>, tockloader_proto::Error>;
        let mut need_reset = false;
        for i in 0..rx_len {

            // if self.pinged.get() == true {
            //     if buffer[i] == 0xfc {
            //         self.led.clear();
            //     }
            // }

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
                    self.buffer.replace(buffer);
                    break;
                }
                Ok(Some(tockloader_proto::Command::ReadRange{address, length})) => {
                    self.state.set(state::ReadRange{address, length, remaining_length: length});
                    self.buffer.replace(buffer);
                    self.page_buffer.take().map(move |page| {
                        let page_size = page.as_mut().len();
                        self.flash.read_page(address as usize / page_size, page);
                    });
                    break;
                }
                Ok(Some(tockloader_proto::Command::ErasePage{address})) => {
                    self.state.set(state::ErasePage);
                    self.buffer.replace(buffer);
                    let page_size = self.page_buffer.map_or(512, |page| { page.as_mut().len() });
                    self.flash.erase_page(address as usize / page_size);
                    break;
                }
                Ok(Some(tockloader_proto::Command::GetAttr{index})) => {
                    self.state.set(state::GetAttribute{index: index});
                    self.buffer.replace(buffer);
                    self.page_buffer.take().map(move |page| {
                        self.flash.read_page(3 + (index as usize / 8), page);
                    });
                    break;
                }
                Ok(Some(tockloader_proto::Command::SetAttr{index, key, value})) => {
                    self.state.set(state::SetAttribute{index});

                    // Copy the key and value into the buffer so it can be added
                    // to the page buffer when needed.
                    for i in 0..8 {
                        buffer[i] = key[i];
                    }
                    buffer[8] = value.len() as u8;
                    for i in 0..55 {
                        // Copy in the value, otherwise clear to zero.
                        if i < value.len() {
                            buffer[9 + i] = value[i];
                        } else {
                            buffer[9+i] = 0;
                        }
                    }
                    self.buffer.replace(buffer);

                    // Initiate things by reading the correct flash page that
                    // needs to be updated.
                    self.page_buffer.take().map(move |page| {
                        self.flash.read_page(3 + (index as usize / 8), page);
                    });
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
// self.led.clear();
                    // Some(tockloader_proto::Response::InternalError)
                    break;
                }
            };


        }

        if need_reset {
            // self.led.clear();
            self.pinged.set(true);
            decoder.reset();

            self.buffer.take().map(|buffer| {
                // self.uart.receive(buffer, 3);
                self.uart.receive_automatic(buffer, 250);
            });

        }
    }
}

impl<'a, U: hil::uart::UARTAdvanced + 'a, F: hil::flash::Flash + 'a, G: hil::gpio::Pin + 'a> hil::flash::Client<F> for Bootloader<'a, U, F, G> {
    fn read_complete(&self, pagebuffer: &'static mut F::Page, _error: hil::flash::Error) {


        match self.state.get() {

            // We just read the correct page for this attribute. Copy it to
            // the out buffer and send it back to the client.
            state::GetAttribute{index} => {

                self.state.set(state::Idle);
                self.buffer.take().map(move |buffer| {
                    buffer[0] = ESCAPE_CHAR;
                    buffer[1] = RES_GET_ATTR;
                    let mut j = 2;
                    for i in 0..64 {
                        let b = pagebuffer.as_mut()[(((index as usize)%8)*64) + i];
                        if b == ESCAPE_CHAR {
                            // Need to escape the escape character.
                            buffer[j] = ESCAPE_CHAR;
                            j += 1;
                        }
                        buffer[j] = b;
                        j += 1;
                    }

                    self.page_buffer.replace(pagebuffer);
                    self.uart.transmit(buffer, j);
                });
            }

            // We need to update the page we just read with the new attribute,
            // and then write that all back to flash.
            state::SetAttribute{index} => {
                self.buffer.map(move |buffer| {
                    // Copy the first 64 bytes of the buffer into the correct
                    // spot in the page.
                    let start_index = ((index as usize)%8)*64;
                    for i in 0..64 {
                        pagebuffer.as_mut()[start_index + i] = buffer[i];
                    }
                    self.flash.write_page(3 + (index as usize / 8), pagebuffer);
                });
            }

            // Pass what we have read so far to the client.
            state::ReadRange{address, length, remaining_length} => {
                // Take what we need to read out of this page and send it
                // on uart. If this is the first message be sure to send the
                // header.
                self.buffer.take().map(move |buffer| {
                    let mut index = 0;
                    if length == remaining_length {
                        buffer[0] = ESCAPE_CHAR;
                        buffer[1] = RES_READ_RANGE;
                        index = 2;
                    }

                    let page_size = pagebuffer.as_mut().len();
                    // This will get us our offset into the page.
                    let page_index = address as usize % page_size;
                    // Length is either the rest of the page or how much we have left.
                    let len = cmp::min(page_size - page_index, remaining_length as usize);
                    // Make sure we don't overflow the buffer.
                    let copy_len = cmp::min(len, buffer.len()-index);

                    // Copy what we read from the page buffer to the user buffer.
                    // Keep track of how much was actually copied.
                    let mut actually_copied = 0;
                    for i in 0..copy_len {
                        // Make sure we don't overflow the buffer. We need to
                        // have at least two open bytes in the buffer
                        if index >= (buffer.len() - 1) {
                            break;
                        }

                        // Normally do the copy and check if this needs to be
                        // escaped.
                        actually_copied += 1;
                        let b = pagebuffer.as_mut()[page_index + i];
                        if b == ESCAPE_CHAR {
                            // Need to escape the escape character.
                            buffer[index] = ESCAPE_CHAR;
                            index += 1;
                        }
                        buffer[index] = b;
                        index += 1;
                    }

                    // Update our state.
                    let new_address = address as usize + actually_copied;
                    let new_remaining_length = remaining_length as usize - actually_copied;
                    self.state.set(state::ReadRange{address: new_address as u32, length, remaining_length: new_remaining_length as u16});

                    // And send the buffer to the client.
                    self.page_buffer.replace(pagebuffer);
                    self.uart.transmit(buffer, index);
                });
            }

            _ => {}
        }




    }

    fn write_complete(&self, pagebuffer: &'static mut F::Page, _error: hil::flash::Error) {
        self.page_buffer.replace(pagebuffer);

        match self.state.get() {

            // Attribute writing done, send an OK response.
            state::SetAttribute{index: _} => {
                self.state.set(state::Idle);
                self.buffer.take().map(move |buffer| {
                    buffer[0] = ESCAPE_CHAR;
                    buffer[1] = RES_OK;
                    self.uart.transmit(buffer, 2);
                });
            }

            _ => {
                self.buffer.take().map(|buffer| {
                    self.uart.receive_automatic(buffer, 250);
                });

            }
        }
    }

    fn erase_complete(&self, _error: hil::flash::Error) {
        match self.state.get() {

            // Page erased, return OK
            state::ErasePage => {
                self.state.set(state::Idle);
                self.buffer.take().map(move |buffer| {
                    buffer[0] = ESCAPE_CHAR;
                    buffer[1] = RES_OK;
                    self.uart.transmit(buffer, 2);
                });
            }

            _ => {
                self.buffer.take().map(|buffer| {
                    self.uart.receive_automatic(buffer, 250);
                });

            }
        }
    }
}
