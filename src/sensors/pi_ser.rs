// Container for serial sensor management through a Pi
// Use test build from RPiSerialTesting to see working example using serialport
// Use documentation from the following to help build out
// mio-serial: https://docs.rs/mio-serial/latest/mio_serial/index.html
// tokio-serial: https://docs.rs/tokio-serial/latest/tokio_serial/

use tokio_serial::{self, SerialPortBuilderExt};
use tokio_util::codec::{Decoder, Encoder};
use futures::stream::StreamExt;
use bytes::BytesMut;
use std::path::Path;
use std::io;
use super::*;

// Note to future self
// Keep the parsing of the sensor data here and only push them into the objects from the library when they are ready
// Each communication type is going to have its own way of parsing the input into the correct object

/// adapted from tokio_serial example
struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return match String::from_utf8(line.as_ref().to_vec()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            };
        }
        Ok(None)
    }
}

impl Encoder<String> for LineCodec {
    type Error = io::Error;

    fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}


// Structure to contain serial messages and their information
#[derive(Clone, Debug)]
pub struct SerialMsg {
    source: Uuid,
    humidity: Option<i32>,
    temp_cel: Option<f32>,
    temp_fah: Option<f32>,
    presence: Option<bool>,
    threshol: Option<bool>
}

impl SerialMsg {
    pub fn parse_str_to_msg(msg: String) -> Result<SerialMsg, tokio_serial::Error> {
        let msg: &str = msg.trim();
        let components: Vec<&str> = msg.split('#').collect();
        let vec_length: usize = components.len();
        if vec_length < 6 || vec_length >= 7 {
            let err_msg: String = format!("Too many or too few items recovered to make a SerialMsg. Received: {:#?}", components);
            return Err(tokio_serial::Error { kind: tokio_serial::ErrorKind::InvalidInput, description: err_msg });
        }
        let source_uuid: Uuid = match Uuid::parse_str(components[0]) {
            Ok(source) => source,
            Err(errored) => return Err(tokio_serial::Error{ kind: tokio_serial::ErrorKind::InvalidInput, description: errored.to_string() })
        };

        Ok(SerialMsg { source: source_uuid, humidity: parse_str_to_int(components[1])?,
                        temp_cel: parse_str_to_float(components[2])?,
                        temp_fah: parse_str_to_float(components[3])?,
                        presence: parse_str_to_bool(components[4])?,
                        threshol: parse_str_to_bool(components[5])? })
    }
    
}

fn parse_str_to_bool(item: &str) -> Result<Option<bool>, tokio_serial::Error> {
    let absent_check: bool = match parse_item_for_absent(item) {
        Ok(_) => true,
        Err(_) => false
    };
    if absent_check {
        return Ok(None)
    } else {
        let real_bool: Result<bool, std::str::ParseBoolError>  = item.parse::<bool>();
        match real_bool {
            Ok(boolean) => return Ok(Some(boolean)),
            Err(e) => return Err(tokio_serial::Error { kind: tokio_serial::ErrorKind::InvalidInput, description: e.to_string() })
        }
    }
}

fn parse_str_to_int(item: &str) -> Result<Option<i32>, tokio_serial::Error> {
    let absent_check: bool = match parse_item_for_absent(item) {
        Ok(_) => true,
        Err(_) => false
    };
    if absent_check {
        return Ok(None)
    } else {
        let real_check: Result<i32, std::num::ParseIntError>  = item.parse::<i32>();
        match real_check {
            Ok(res) => return Ok(Some(res)),
            Err(e) => return Err(tokio_serial::Error { kind: tokio_serial::ErrorKind::InvalidInput, description: e.to_string() })
        }
    }
}

fn parse_str_to_float(item: &str) -> Result<Option<f32>, tokio_serial::Error> {
    let absent_check: bool = match parse_item_for_absent(item) {
        Ok(_) => true,
        Err(_) => false
    };
    if absent_check {
        return Ok(None)
    } else {
        let real_check: Result<f32, std::num::ParseFloatError>  = item.parse::<f32>();
        match real_check {
            Ok(res) => return Ok(Some(res)),
            Err(e) => return Err(tokio_serial::Error { kind: tokio_serial::ErrorKind::InvalidInput, description: e.to_string() })
        }
    }
}

fn parse_item_for_absent(item: &str) -> Result<bool, tokio_serial::Error> {
    let test_char: Result<char, std::char::ParseCharError> = item.parse::<char>();
    match test_char {
        Ok('A') => return Ok(true),
        Ok('a') => return Ok(true),
        _ => return Err(tokio_serial::Error { kind: tokio_serial::ErrorKind::InvalidInput, description: "Got invalid character in message".to_string() })
    }
}

/// Create a serial port with the desired defaults
/// Optionally will accept a path to another serial device
/// Make this mutable if you'd like to modify settings
/// Default is /dev/ttyAMA0 at 9600 baud with 8 data bits, no parity, and 1 stop bit
/// # Error
/// If the path provided or the default path can't be found, this will error out with NoDevice
pub fn build_serial(path: Option<&str>) -> Result<tokio_serial::SerialPortBuilder, tokio_serial::Error> {
    let test_path = match path {
        Some(tpath) => tpath,
        None => "/dev/ttyAMA0",
    };
    if Path::new(test_path).exists() {
        Ok(tokio_serial::new(test_path, 9600)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One))
    } else {
        let msg: String = format!("Unable to find device: {}", test_path);
        Err(tokio_serial::Error { kind: tokio_serial::ErrorKind::NoDevice, description: msg })
    }
}

/*
MAX 485 wiring reminder:
Pin Name	Description
VCC	        This is the power supply pin. It is connected with 5V that powers up the module.
A	        This is the non-inverting receiver input and driver output. It is connected with A on the other module.
B	        This is the inverting receiver input and driver output. It is connected with B on the other module.
GND	        This is the GND pin. It is connected with common ground.
RO	        This is the receiver output pin. It is connected with the RX pin of the microcontroller.
RE	        This is the receiver output enable pin. To enable, it is set at a LOW state.
DE	        This is the driver output enable pin. To enable, it is set at a HIGH state.
DI	        This is the driver input. It is connected with the TX pin of the microcontroller.

Thus wiring is:
Module     Device
VCC <----> 5V
GND <----> GND
RE+DE <--> Digital Output (LOW to receive, HIGH to send)
RO <-----> UART RX
DI <-----> UART TX

---
Protocol Ideas
- Received messages from sensors should start with a valid UUID
- Valid commands to established sensors should start with hash of UUID
- New sensors requesting association send all zero'd UUID in the correct tranmission showing capability
- New sensor format: UUID#HumidityBool#TempCBool#TempFBool#PresenceBool#ThresholdBool\n
- Command format: $UUID#Command#Arg1#Arg2#Arg3#Arg4\n
- The number of # must be fixed but areas in between #s do not need anything if not needed for the command
- Command ideas: Provision, Set, Delete/Ban
- Provision Command: $PRO#UUID#SALT###\n
- Set frequency Command: $UUID#SET#delay#timeINmilliseconds##\n
- Set activity Command: $UUID#SET#active#bool##\n
*/

/// Grab a serial message if available
#[allow(unused_mut)]
pub async fn get_serial_msg(ser_built: tokio_serial::SerialPortBuilder) -> Result<Option<SerialMsg>, tokio_serial::Error> {
    let mut my_open_serial: tokio_serial::SerialStream = ser_built.open_native_async()?;

    #[cfg(unix)]
    match my_open_serial.set_exclusive(false) {
        Ok(_) => (),
        Err(_) => println!("Tried and failed to set the port as not exclusive!"),
    }

    let mut reader: tokio_util::codec::Framed<tokio_serial::SerialStream, LineCodec> = LineCodec.framed(my_open_serial);

    let line_result: String = match reader.next().await {
        None => return Ok(None),
        Some(line) => line?,
    };

    return Ok(Some(SerialMsg::parse_str_to_msg(line_result)?))
}

