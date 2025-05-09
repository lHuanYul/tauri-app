use std::io::Write;
use std::time::Duration;
use log::{error, info, warn};
use serialport::SerialPort;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio::io::AsyncReadExt;
use crate::mods::packet_mod;
use packet_mod::UartPacket;

/// A manager for serial port communication.
pub struct PortManager {
    /// The serial port instance; `None` indicates the port is not open.
    port: Option<Box<dyn SerialPort>>,
    port_name: Option<String>,
}

impl PortManager {
    /// Creates a new `PortManager` with no open port.
    pub fn new() -> Self {
        Self {
            port: None,
            port_name: None,
        }
    }

    /// Ensures the port is open, returning an error if not.
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` containing an error message if the
    /// serial port is not yet opened.
    fn check_open(&self) -> Result<(), String> {
        if self.port.is_none() {
            let message = format!("Port not openned");
            error!("{}", message);
            return Err(message)
        }
        Ok(())
    }

    /// Opens the serial port with the specified settings.
    ///
    /// # Arguments
    ///
    /// * `port_name` - The name of the serial port (e.g., "COM4").
    /// * `baudrate` - The baud rate for the connection.
    /// * `timeout_ms` - The timeout in milliseconds.
    ///
    /// # Returns
    ///
    /// A `Result` containing a success message or an error string.
    pub fn open(&mut self, port_name: &str, baudrate: u32, timeout_ms: u64) -> Result<String, String> {
        if self.port.is_some() {
            let message = format!("Port already openned: {}", self.port_name.as_ref().unwrap());
            error!("{}", message);
            return Err(message);
        }
        let port = serialport::new(port_name, baudrate)
            .timeout(Duration::from_millis(timeout_ms))
            .open()
            .map_err(|e| {
                let message = format!("Port open failed: {}", e);
                error!("{}", message);
                message
            })?;
        self.port = Some(port);
        self.port_name = Some(port_name.to_string());
        let message = format!("Port open succeeded: {}", port_name);
        info!("{}", message);
        Ok(message)
    }

    /// Closes the currently open serial port.
    ///
    /// # Returns
    ///
    /// A `Result` containing a success message or an error string if no port was open.
    pub fn close(&mut self) -> Result<String, String> {
        if self.port.is_none() {
            let message = "Port already closed".to_string();
            warn!("{}", message);
            return Err(message);
        }
        self.port = None;
        self.port_name = None;
        let message = "Port close succeeded".to_string();
        info!("{}", message);
        Ok(message)
    }

    /// Writes a single byte to the serial port.
    ///
    /// # Arguments
    ///
    /// * `data` - A `u8` value to write.
    ///
    /// # Returns
    ///
    /// A `Result` containing a success message or an error string.
    pub fn write(&mut self, data: u8) -> Result<String, String> {
        self.check_open()?;
        let port = self.port.as_mut().unwrap();

        port.write_all(&[data]).map_err(|e| {
            let message = format!("Port write failed: {}", e);
            error!("{}", message);
            message
        })?;
        let message = format!("Port write succeeded: {:?} (0x{:02X})", data, data);
        info!("{}", message);
        Ok(message)
    }

    /// Writes a `UartPacket` to the serial port.
    ///
    /// This method converts the `UartPacket` into a 10-byte buffer and writes it out.
    ///
    /// # Arguments
    ///
    /// * `packet` - A `UartPacket` instance to send.
    ///
    /// # Returns
    ///
    /// A `Result` containing a success message (with packet information) or an error string.
    pub fn write_packet(&mut self, packet: UartPacket) -> Result<String, String> {
        self.check_open()?;
        let port = self.port.as_mut().unwrap();

        let buffer = packet.unpack()?;
        port.write_all(&buffer).map_err(|e| {
            let message = format!("Port write failed: {}", e);
            error!("{}", message);
            message
        })?;

        let packet_info = packet.read("Port write succeeded:")?;
        Ok(format!("{}", packet_info))
    }

    /// Reads 10 bytes from the serial port and parses them into a `UartPacket`.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `UartPacket` if successful, or an error string.
    pub fn read_packet(&mut self) -> Result<UartPacket, String> {
        self.check_open()?;
        let port = self.port.as_mut().unwrap();

        let mut buf = [0u8; 10];
        port.read_exact(&mut buf).map_err(|e| {
            let message = format!("Port read failed: {}", e);
            error!("{}", message);
            message
        })?;
        info!("Port read succeeded:\n    {:?}", buf);
        let packet = UartPacket::pack(&buf)?;
        Ok(packet)
    }
}
