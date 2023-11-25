// Container for serial sensor management through a Pi

use futures::SinkExt;
use tokio_serial::{self, SerialPortBuilderExt};
use tokio_util::codec::{Decoder, Encoder};
use futures::stream::StreamExt;
use bytes::BytesMut;
use std::path::Path;
use std::{io, fmt::Display};
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
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return Ok(Some(line.as_ref().to_vec()));
        }
        Ok(None)
    }
}

impl Encoder<Vec<u8>> for LineCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let item_size: usize = item.len();
        dst.reserve(5 + item_size);
        let byte_version: &[u8] = &item;
        dst.extend_from_slice(byte_version);
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

/// Structure for containing raw serial messages and parsing them
pub struct SerialMsg {
    orig_msg: Vec<u8>,
    parsed_iv: Option<[u8; 16]>,
    parsed_payload: Option<Vec<u8>>,
    parsed_hmac: Option<[u8; 32]>,
    decrypted_payload: Option<String>,
}

impl SerialMsg {
    /// Creating a new SerialMsg requires the UUID that this is coming from and a String of what the serial interface created.
    /// This will attempt to breakdown the message, confirm validity using HMAC and then decrypt the message inside the payload.
    /// For potential troubleshooting, the original message used to parse the items is kept
    /// # Errors
    /// If the String is just too short to possibly be a valid communication from a registered sensor, this will fail
    /// This will NOT FAIL if the UUID, IV or HMAC are wrong for the payload to decrypt successfully
    pub fn new(source: &Uuid, msg: Vec<u8>) -> Result<SerialMsg, tokio_serial::Error> {
        let byte_buffer: &[u8] = &msg;

        if byte_buffer.len() < 50 {
            return Err(tokio_serial::Error { kind: tokio_serial::ErrorKind::InvalidInput,
                description: "String provided is not large enough to have valid IV, payload, and HMAC!".to_string() });
        }

        let mut iv: [u8; 16] = [0; 16];
        let mut given_hmac: [u8; 32] = [0; 32];
        let mut given_payload: Vec<u8> = vec![];
        let hmac_start: usize = byte_buffer.len() - 32;

        iv.copy_from_slice(&byte_buffer[0..15]);
        given_hmac.copy_from_slice(&byte_buffer[hmac_start..]);

        for bit in &byte_buffer[16..hmac_start] {
            given_payload.push(bit.clone());
        }

        let source_uuid: Uuid = source.clone();
        let mut decrypted: Option<String> = None;
        if verify_hmac(source_uuid, &given_payload, given_hmac) {
            decrypted = match decrypt_message(source_uuid, iv, &given_payload) {
                Ok(inner) => Some(inner),
                Err(_) => None,
            };
        }
        
        Ok(SerialMsg {
            orig_msg: msg.clone(),
            parsed_iv: Some(iv),
            parsed_payload: Some(given_payload),
            parsed_hmac: Some(given_hmac),
            decrypted_payload: decrypted,
        })
    }
    /// Clones the original vector used to generate the SerialMsg.
    /// Only helpful for troubleshooting
    pub fn get_org_msg(&self) -> Vec<u8> {
        self.orig_msg.clone()
    }
    /// Clones the IV parsed from the message. These should be reasonably random!
    pub fn get_iv(&self) -> Option<[u8; 16]> {
        self.parsed_iv.clone()
    }
    /// Clones the HMAC appended at the end of the payload
    pub fn get_hmac(&self) -> Option<[u8; 32]> {
        self.parsed_hmac.clone()
    }
    /// Clones the encrypted payload from the message
    pub fn get_payload(&self) -> Option<Vec<u8>> {
        self.parsed_payload.clone()
    }
    /// Checks if the payload has been decrypted but not if it was successful
    pub fn payload_decrypted(&self) -> bool {
        match self.decrypted_payload {
            Some(_) => return true,
            None => return false,
        }
    }
    /// Verifies that the payload in the SerialMsg and the HMAC in the same SerialMsg match based on the provided UUID.
    /// This will return FALSE if the payload is empty, if the HMAC is empty, if the UUID is incorrect or if the HMAC indicates the message does not match
    pub fn verify_payload(&self, source: &Uuid) -> bool {
        verify_hmac(source.clone(), &self.get_payload().unwrap_or(vec![0]), self.get_hmac().unwrap_or([0; 32]))
    }

    /// Verifies that the payload is decrypted successfully by checking the contained UUID.
    /// This will return false if the message does not start with the UUID provided of if the payload was not decrypted.
    /// A failure can indiciate either a bad IV, an incorrect UUID or a damaged/incomplete/tampered payload
    pub fn verify_decryption(&self, source: &Uuid) -> bool {
        if self.payload_decrypted() {
            let local_message = self.decrypted_payload.clone().unwrap();
            if local_message.starts_with(&source.as_simple().to_string()) {
                return true;
            } else {
                return false;
            }
        } else {
            return false;
        }
    }

    /// Get the String that was decrypted
    pub fn get_decrypted_payload(&self) -> Option<String> {
        self.decrypted_payload.clone()
    }

    /// Create a SerialReading based on info in the decrypted payload
    /// #Errors
    /// Will return an "Unknown" tokio_serial error with a description of what happened if this cannot make a SerialReading
    pub fn generate_serial_reading(&self, source: &Uuid) -> Result<SerialReading, tokio_serial::Error> {
        if self.verify_payload(source) {
            if self.verify_decryption(source) {
                SerialReading::parse_str_to_msg(self.get_decrypted_payload().unwrap())
            } else {
                return Err(tokio_serial::Error { kind: tokio_serial::ErrorKind::Unknown, description: "Payload failed content verification.".to_string() });
            }
        } else {
            return Err(tokio_serial::Error { kind: tokio_serial::ErrorKind::Unknown, description: "Payload failed HMAC verification.".to_string() });
        }
    }
}

/// Structure to contain serial readings and their information
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
    pub fn new(source: Uuid) -> SerialReading {
        SerialReading { source: source,
                humidity: None, temp_cel: None, temp_fah: None, presence: None, threshol: None }
    }
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

impl Display for SerialReading {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut display_message: String = String::new();
        if let Some(humid) = self.humidity {
            display_message = format!("{}Humidity: {}|", display_message, humid)
        };
        if let Some(temp) = self.temp_cel {
            display_message = format!("{}Celsius: {}|", display_message, temp)
        };
        if let Some(temp) = self.temp_fah {
            display_message = format!("{}Fahrenheit: {}|", display_message, temp)
        };
        if let Some(pres) = self.presence {
            display_message = format!("{}Presense Detected: {}|", display_message, pres)
        };
        if let Some(thresh) = self.threshol {
            display_message = format!("{}Threshold Open: {}|", display_message, thresh)
        };
        write!(f, "UUID: {}, Readings: |{}", self.source, display_message)
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
fn build_serial(path: Option<&str>) -> Result<tokio_serial::SerialPortBuilder, tokio_serial::Error> {
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

/* Protocol/Max485 notes
Wiring:
Module     Device
VCC <----> 5V
GND <----> GND
RE+DE <--> Digital Output (LOW to receive, HIGH to send)
RO <-----> UART RX
DI <-----> UART TX

---
Protocol

New sensors requesting association send all zero'd UUID in correct format showing capability in plaintext
- New sensor format: UUID#HumidityBool#TempCBool#TempFBool#PresenceBool#ThresholdBool\n
When this information reaches the server, the server generates a UUID and saves the capabilities in the database.
Once the DB save is confirmed, the server generates a SHA256 hash of the UUID and relies with the UUID and confirmed capabilities.
- New sensor response if accepted: UUIDinSHA256\n
- If not accepted (capabilities look wrong) then the process starts over
-- UUID will time out due to invalid response and will be banned by server --
Server compares given SHA256 with the SHA256 generated to ensure the sensor is generating SHA256 correctly and has the right UUID.
-If the SHA256 is wrong or a timeout is reached waiting for the hash from the sensor, the UUID is banned and the server will ignore messages with that UUID.
-If the hash is correct, the server waits for the sensor to send the sample encrypted payload to ensure the encryption is working.
-- Payload should be a copy of what the server sent EG UUID#BOOL#BOOL#BOOL#BOOL#BOOL

-----
Example of good startup:
Sensor: 00000000000000000000000000000000#TRUE#TRUE#TRUE#FALSE#FALSE\n
Server: 0f3832c6201547c9a296962fd94b3e38#TRUE#TRUE#TRUE#FALSE#FALSE\n
Sensor: 23126bb7af6b98b07f0388c7b0a4008a490f2fc4f60fe9ae46288aebaca65858\n
*All traffic afterwards will be encrypted and have a structure of IV|PAYLOAD|HMAC without separators*
Sensor: **Encrypted payload showing server config response to ensure encryption is valid**
-----

- The number of # must be fixed inside the payload
-- For commands, #s can be trailing with no items inside (see examples below)
-- For sensors, the #s need to have A if not used
- Set frequency Command: SET#delay#timeINmilliseconds##
- Set activity Command: SET#active#bool##
- sensor reading example: 0f3832c6201547c9a296962fd94b3e38#A#A#A#FALSE#A
*/

/// Take a UUID, an IV and the message to encrypt and create a Vector of bytes that represents that encrypted message
fn encrypt_message(dest: Uuid, iv: [u8; 16], mess: &String) -> Vec<u8> {
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
fn decrypt_message(source: Uuid, iv: [u8; 16], payload: &Vec<u8>) -> Result<String, std::string::FromUtf8Error> {
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
fn generate_hmac(the_uuid: Uuid, payload: &Vec<u8>) -> Result<[u8; 32], InvalidLength> {
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
fn verify_hmac(the_uuid: Uuid, payload: &Vec<u8>, received_hmac: [u8; 32]) -> bool {
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

/// Structure to contain serial communication for one device
/// For now, we are assuming 1:1 sensor to serial interface
#[derive(Clone, Debug)]
pub struct SerialInterface {
    port: tokio_serial::SerialPortBuilder,
    uid: Uuid,
}

impl Default for SerialInterface {
    fn default() -> SerialInterface {
        SerialInterface { port: build_serial(None).unwrap(), uid: Uuid::nil() }
    }
}

impl Display for SerialInterface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UUID: {}, Port Info: {:?}", self.uid.as_hyphenated().to_string(), self.port)
    }
}

impl SerialInterface {
    pub fn new(port_path: &str, set_uuid: Option<Uuid>) -> Result<SerialInterface, tokio_serial::Error> {
        let serial_port = build_serial(Some(port_path))?;
        match set_uuid {
            Some(uid_set) => Ok(SerialInterface { port: serial_port, uid: uid_set }),
            None => Ok(SerialInterface { port: serial_port, uid: Uuid::nil() })
        }
    }

    #[allow(unused_mut)]
    pub async fn get_message(&self) -> Result<Option<SerialMsg>, tokio_serial::Error> {
        let local_port: tokio_serial::SerialPortBuilder = self.port.clone();
        let mut open_stream: tokio_serial::SerialStream = local_port.open_native_async()?;

        #[cfg(unix)]
        match open_stream.set_exclusive(false) {
            Ok(_) => (),
            Err(_) => println!("Tried and failed to set the port as not exclusive!"),
        }

        let mut reader: tokio_util::codec::Framed<tokio_serial::SerialStream, LineCodec> = LineCodec.framed(open_stream);

        let line_result: Result<Vec<u8>, io::Error> = match reader.next().await {
            None => return Ok(None),
            Some(line) => line,
        };
        if line_result.is_err() {
            return Err(tokio_serial::Error { kind: tokio_serial::ErrorKind::Io(line_result.unwrap_err().kind()),
                    description: format!("Error reading stream while using: {:?}", self.port) })
        } else {
            let serial_msg = SerialMsg::new(&self.uid, line_result.unwrap())?;
            return Ok(Some(serial_msg))
        }
    }

    #[allow(unused_mut)]
    pub async fn send_command(&self, com: String) -> Result<(), tokio_serial::Error> {
        let local_port: tokio_serial::SerialPortBuilder = self.port.clone();
        let mut open_sink: tokio_serial::SerialStream = local_port.open_native_async()?;

        #[cfg(unix)]
        match open_sink.set_exclusive(false) {
            Ok(_) => (),
            Err(_) => println!("Tried and failed to set the port as not exclusive!"),
        }

        let pending_command: Result<Vec<u8>, InvalidLength> = SerialCmd::new(self.uid.clone(), None, com.clone()).build_msg();
        if pending_command.is_err() {
            return Err(tokio_serial::Error { kind: tokio_serial::ErrorKind::InvalidInput,
                    description: format!("Encryption and hashing of command failed. Command issued: {}", com) });
        } else {
            let message: Vec<u8> = pending_command.unwrap();
            let message_size: usize = message.len();
            let mut writer: tokio_util::codec::Framed<tokio_serial::SerialStream, LineCodec> = LineCodec.framed(open_sink);
            
        }

        Ok(())
    }
}