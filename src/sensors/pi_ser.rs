// Container for serial sensor management through a Pi
// Use test build from RPiSerialTesting to see working example using serialport
// Use documentation from the following to help build out
// mio-serial: https://docs.rs/mio-serial/latest/mio_serial/index.html
// tokio-serial: https://docs.rs/tokio-serial/latest/tokio_serial/

use tokio_serial;
use tokio_serial::SerialPort;
use std::path::Path;
use std::io::{BufReader, BufRead};
use super::*;

// Note to future self
// Keep the parsing of the sensor data here and only push them into the objects from the library when they are ready
// Each communication type is going to have its own way of parsing the input into the correct object

// Structure to contain serial messages and their information
#[derive(Clone, Debug)]
struct SerialMsg {
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

/// check for the desired default which is /dev/ttyAMA0
/// This currently doesn't seem to return anything even though you can manually pick a serial port and use it just fine
/// # Error
/// If trying to look at the serial ports errors, this will return that error instead of a bool
#[allow(dead_code)]
pub fn check_for_pi_uart() -> Result<bool, tokio_serial::Error> {
    let available_ports: Vec<tokio_serial::SerialPortInfo> = tokio_serial::available_ports()?;
    for sport in available_ports.into_iter() {
        if sport.port_name == "/dev/ttyAMA0".to_string() {
            return Ok(true)
        }
    }
    return Ok(false)
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


// Note, turn this into an async loop to constantly eat serial info and prep it for the database
// This was copied from my test with just changing the function naming so it will need tweaking
fn grab_serial_data() -> Result<(), tokio_serial::Error> {
    let my_serial: tokio_serial::SerialPortBuilder = build_serial(None)?;

    let my_open_serial: Box<dyn SerialPort> = my_serial.open().expect("Unable to open port!");

    match my_open_serial.clear(tokio_serial::ClearBuffer::All) {
        Ok(_) => println!("Buffer cleared."),
        Err(e) => println!("Encountered an error clearing the buffers: {:#?}", e),
    }

    let mut msg: String = String::new();

    let mut reader: BufReader<Box<dyn SerialPort>> = BufReader::new(my_open_serial);

    let mut enumer: u8 = 0;

    while enumer < 90 {
        let result: Result<usize, std::io::Error> = reader.read_line(&mut msg);

        if result.is_ok() {
            let new_reading: SerialMsg = SerialMsg::parse_str_to_msg(msg)?;

            println!("Received and parsed SerialMsg as: {:#?}", new_reading);

            reader.consume(result.unwrap());
            msg = "".to_string();

            println!("Cleared buffers and waiting for next message.");
        } else {

            println!("Nothing in buffers. Waiting for another message...");
            std::thread::sleep(std::time::Duration::from_secs(2));

        }
        enumer = enumer + 1;
    }

    Ok(())
}