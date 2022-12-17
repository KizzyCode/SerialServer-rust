//! A unified server

use crate::{config::Config, error::Error, logger::Logger, serial::SerialDevice};
use std::{
    net::{ToSocketAddrs, UdpSocket},
    thread,
};

/// The server
pub struct Server {
    /// The server config
    config: Config,
    /// The UDP socket
    socket: UdpSocket,
    /// The serial device
    serial: SerialDevice,
    /// The logger
    logger: Option<Logger>,
}
impl Server {
    /// Creates a new server
    pub fn new(config: Config) -> Result<Self, Error> {
        // Setup socket
        let socket = UdpSocket::bind(&config.udp.listen)?;

        // Setup spipe and logger
        let serial = SerialDevice::new(&config.serial.device, config.serial.baudrate)?;
        let logger = config.log.enabled.then(Logger::new);
        Ok(Self { config, socket, serial, logger })
    }

    /// Starts the server runloop
    pub fn runloop(self) -> Result<(), Error> {
        thread::scope(|scope| -> Result<(), Error> {
            // Clone serial port and spawn threads
            let (serial_in, serial_out) = (self.serial.try_clone()?, self.serial.try_clone()?);
            let serial2udp = scope.spawn(|| self.runloop_serial2udp(serial_in));
            let udp2serial = scope.spawn(|| self.runloop_udp2serial(serial_out));

            // Wait for threads and propagate results
            serial2udp.join().expect("Serial->UDP thread has panicked")?;
            udp2serial.join().expect("UDP->serial thread has panicked")?;
            Ok(())
        })
    }
    /// The serial->UDP runloop
    fn runloop_serial2udp(&self, mut serial: SerialDevice) -> Result<(), Error> {
        // Unwrap and resolve the remote address and create the socket
        let maybe_address_socket = 'make_socket: {
            // Unwrap the address
            let Some(address_string) = &self.config.udp.send else {
                break 'make_socket None;
            };

            // Parse the address
            let Ok(mut addresses) = address_string.to_socket_addrs() else {
                break 'make_socket None;
            };
            let Some(address) = addresses.next() else {
                break 'make_socket None;
            };

            // Create the socket
            let socket = UdpSocket::bind(address)?;
            socket.set_ttl(self.config.udp.ttl)?;
            Some((address, socket))
        };

        // The `socket::send_to` implementation *if there is a remote address configured*
        let socket_send_to = move |buf: &[u8]| -> Result<usize, Error> {
            // Send UDP packet if a multicast address is defined or perform a no-op
            let sent = match maybe_address_socket.as_ref() {
                Some((address, socket)) => socket.send_to(buf, address)?,
                None => buf.len(),
            };
            Ok(sent)
        };

        // Send the packets
        let mut buf = vec![0; 400];
        loop {
            // Receive serial chunk
            let bytes_read = serial.read(&mut buf)?;
            if bytes_read > 0 {
                // Send the message to the multicast address if a multicast
                socket_send_to(&buf[..bytes_read])?;
                self.log(&buf[..bytes_read]);
            }
        }
    }
    /// The UDP->serial runloop
    fn runloop_udp2serial(&self, mut serial: SerialDevice) -> Result<(), Error> {
        let mut buf = vec![0; 4000];
        loop {
            // Receive UDP packet
            let bytes_read = self.socket.recv(&mut buf)?;
            if bytes_read > 0 {
                // Write the message to the serial device
                serial.write_all(&buf[..bytes_read])?;
                serial.flush()?;
                self.log(&buf[..bytes_read]);
            }
        }
    }

    /// Logs the data if there is a logger available
    fn log(&self, data: &[u8]) {
        // Unwrap the logger if available
        if let Some(logger) = self.logger {
            // Log the data
            logger.log(data);
        }
    }
}
