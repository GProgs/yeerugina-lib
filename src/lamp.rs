use log::debug;

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

use crate::cmd::Command;

#[derive(Debug)]
/// A struct that represents a Yeelight lamp.
///
/// The struct implements Read and Write,
/// so you can send commands by using the write! macro as follows:
/// ```rust
/// lamp.send_cmd(cmd)?;
/// // calls inside itself:
/// write!(&mut lamp, "{}\r\n", cmd)?;
/// ```
pub struct Lamp {
    /// The connection to the lamp.
    ///
    /// For changing properties such as read and write timeouts, call the methods on this field directly.
    pub stream: TcpStream,
}
// TcpStream will be dropped once we go out of scope

impl Lamp {
    /// Create a new Lamp from an IP address (or several addresses).
    ///
    /// The argument can be anything that implements [`ToSocketAddrs`], such as String, &str, or (&str, u16).
    /// You can pass multiple addresses into the method, and the TcpStream will use the first successful connection.
    /// If no address provides a connection, the most recent (i.e. last) error will be returned.
    pub fn connect<A: ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
        debug!("Lamp | Attempt connect");
        let stream = TcpStream::connect(addr)?;
        debug!("Lamp | Connection Successful");
        Ok(Self { stream })
    }

    /// Send a command to the lamp.
    ///
    /// This command takes a reference to a [`Command`], so it does not consume the command.
    pub fn send_cmd(&mut self, cmd: &Command) -> std::io::Result<()> {
        //self.stream.write
        debug!("Lamp | Sending command {cmd:?}");
        write!(self, "{}\r\n", cmd)
    }
}

// Delegate reading/writing to the internal stream.
impl Read for Lamp {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for Lamp {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}
