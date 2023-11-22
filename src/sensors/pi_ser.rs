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
use sha2;
use hmac::{Hmac, Mac, digest::InvalidLength};
use libaes::Cipher;
use rand::Rng;
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

/// Stucture for containing a serial message going out
#[derive(Clone, Debug)]
pub struct SerialCmd {
    destination: Uuid,
    iv: [u8; 16],
    command: String,
}

impl SerialCmd {
    pub fn new(dest: Uuid, iv: Option<[u8; 16]>, com: String) -> SerialCmd {
        let final_iv: [u8; 16] = match iv {
            Some(given_iv) => given_iv,
            None => rand::thread_rng().gen::<[u8; 16]>(),
        };
        SerialCmd { destination: dest, iv: final_iv, command: com }
    }
    pub fn build_msg(&self) -> Result<Vec<u8>, InvalidLength> {
        let local_cmd: SerialCmd = self.clone();
        let begin_iv: [u8; 16] = self.iv.clone();
        let payload: Vec<u8> = encrypt_message(local_cmd.destination.clone(), local_cmd.iv, &local_cmd.command);

        let gen_hmac: [u8; 32] = generate_hmac(local_cmd.destination, &payload)?;
        let mut final_payload: Vec<u8> = vec![];
        for bit in begin_iv  {
            final_payload.push(bit)
        }
        for bit in payload.into_iter() {
            final_payload.push(bit)
        }
        for bit in gen_hmac {
            final_payload.push(bit)
        }
        Ok(final_payload)
    }
}

/// Structure to contain serial messages and their information
#[derive(Clone, Debug)]
pub struct SerialReading {
    source: Uuid,
    humidity: Option<i32>,
    temp_cel: Option<f32>,
    temp_fah: Option<f32>,
    presence: Option<bool>,
    threshol: Option<bool>
}

impl SerialReading {
    pub fn parse_str_to_msg(msg: String) -> Result<SerialReading, tokio_serial::Error> {
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

        Ok(SerialReading { source: source_uuid, humidity: parse_str_to_int(components[1])?,
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
Wiring:
Module     Device
VCC <----> 5V
GND <----> GND
RE+DE <--> Digital Output (LOW to receive, HIGH to send)
RO <-----> UART RX
DI <-----> UART TX

---
Protocol Ideas

- New sensors requesting association send all zero'd UUID in the correct tranmission showing capability
- New sensor format: UUID#HumidityBool#TempCBool#TempFBool#PresenceBool#ThresholdBool\n
- New sensor response if accepted: $UUID\n
- Command format: IV|PAYLOADwithCOMMAND|MAC\n
(NOTE: IV will always be 16 bits first, MAC will always be 32 bits last. No separators in communication or you will foul up the encryption )
- The number of # must be fixed inside the payload but areas in between #s do not need anything if not needed for the command
- Command ideas: Set frequency, Set activity
- Set frequency Command: SET#delay#timeINmilliseconds##\n
- Set activity Command: SET#active#bool##\n
*/

/// Grab a serial message if available
#[allow(unused_mut)]
pub async fn get_serial_msg(ser_built: tokio_serial::SerialPortBuilder) -> Result<Option<SerialReading>, tokio_serial::Error> {
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

    return Ok(Some(SerialReading::parse_str_to_msg(line_result)?))
}

/// Take a UUID, an IV and the message to encrypt and create a Vector of bytes that represents that encrypted message
pub fn encrypt_message(dest: Uuid, iv: [u8; 16], mess: &String) -> Vec<u8> {
    let mut new_key: [u8; 32] = [0; 32];
    let mut i: usize = 0;

    for bit in dest.as_simple().to_string().as_bytes() {
        new_key[i] = *bit;
        i=i+1;
    }

    let my_cipher: Cipher = Cipher::new_256(&new_key);
    my_cipher.cbc_encrypt(&iv, mess.as_bytes())
}

/// Take a UUID, an IV and an encrypted message and attempt to decrypt it and convert into a String
/// # Errors
/// If the decrypted bytes can't be made into a string, this will error
/// At this time, this will NOT error if the wrong message was decrypted or decrypted incorrectly!
pub fn decrypt_message(source: Uuid, iv: [u8; 16], payload: &Vec<u8>) -> Result<String, std::string::FromUtf8Error> {
    let mut new_key: [u8; 32] = [0; 32];
    let mut i: usize = 0;

    for bit in source.as_simple().to_string().as_bytes() {
        new_key[i] = *bit;
        i=i+1;
    }

    let my_cipher: Cipher = Cipher::new_256(&new_key);
    let decrypted: Vec<u8> = my_cipher.cbc_decrypt(&iv, &payload[..]);
    String::from_utf8(decrypted)
}

/// Generate an HMAC based on the UUID and a given encrypted payload
pub fn generate_hmac(the_uuid: Uuid, payload: &Vec<u8>) -> Result<[u8; 32], InvalidLength> {
    type HmacSha256 = Hmac<sha2::Sha256>;

    let mut final_mac: [u8; 32] = [0; 32];
    let mut i: usize = 0;

    let mut mac = match HmacSha256::new_from_slice(the_uuid.as_simple().to_string().as_bytes()) {
        Ok(hmacer) => hmacer,
        Err(e) => return Err(e),
    };
    mac.update(&payload.as_slice());
    let result = mac.finalize();
    for bit in result.into_bytes() {
        final_mac[i] = bit;
        i=i+1;
    }
    Ok(final_mac)
}

/// Verify a given UUID and encrypted payload against the extracted HMAC
/// Returns true if there is a match and false if there are any issues
pub fn verify_hmac(the_uuid: Uuid, payload: &Vec<u8>, received_hmac: [u8; 32]) -> bool {
    type HmacSha256 = Hmac<sha2::Sha256>;

    let mut mac = match HmacSha256::new_from_slice(the_uuid.as_simple().to_string().as_bytes()) {
        Ok(hmacer) => hmacer,
        Err(_) => return false,
    };
    mac.update(&payload.as_slice());

    match mac.verify_slice(&received_hmac[..]) {
        Ok(()) => return true,
        Err(_) => return false,
    }
}

