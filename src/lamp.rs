use log::debug;
use pin_project::pin_project;
use smol::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use smol::net::{AsyncToSocketAddrs, TcpStream as AsyncTcpStream};
use smol::pin;

use std::io::{Error, ErrorKind, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

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

    /// Create a new Lamp from an IP address (or several addresses), using a non-zero timeout period.
    ///
    /// As previously, the addr argument can be anything implementing the [`ToSocketAddrs`] trait.
    /// The first successful connection will be used.
    /// If no address provides a connection, the most recent (i.e. last) error will be returned.
    pub fn connect_timeout<A: ToSocketAddrs>(addr: A, timeout: Duration) -> std::io::Result<Self> {
        // Check that timeout is non-zero
        if timeout.is_zero() {
            debug!("Lamp | Zero timeout passed to connect_timeout");
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Non-zero timeout Duration required",
            ));
        }
        debug!("Lamp | Connecting with timeout");
        // Keep track of the most recent error
        // (inspired by std::sys::net::connection::each_addr function, which is used by TcpStream)
        // (see https://doc.rust-lang.org/src/std/sys/net/connection/mod.rs.html)
        let mut last_err = None;
        // Get iterator of socket addresses
        // And try each of them to see what works
        for sock_addr in addr.to_socket_addrs()? {
            // Try to connect
            debug!("Lamp | Attempt connect_timeout");
            let mby_stream = TcpStream::connect_timeout(&sock_addr, timeout);
            match mby_stream {
                Ok(stream) => {
                    debug!("Lamp | Connection with timeout Successful");
                    return Ok(Self { stream });
                }
                Err(e) => last_err = Some(e),
            }
        }
        debug!("Lamp | Connection with timeout Failed");
        match last_err {
            Some(err) => Err(err),
            None => Err(Error::new(ErrorKind::InvalidInput, "No addresses provided")),
        }
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

/// TODO cba to write docs
#[derive(Debug)]
#[pin_project]
pub struct AsyncLamp {
    /// TODO cba to write docs
    #[pin]
    pub stream: AsyncTcpStream,
}

impl AsyncLamp {
    /// TODO cba to write docs
    pub async fn connect<A: AsyncToSocketAddrs>(addr: A) -> std::io::Result<Self> {
        let stream = AsyncTcpStream::connect(addr).await?;
        Ok(Self { stream })
    }

    /// Send a command to the lamp using asynchronous I/O.
    ///
    /// This command takes in a reference to a [`Command`], and returns a Poll containing a Result.
    pub async fn send_cmd(&mut self, cmd: &Command) -> std::io::Result<()> {
        debug!("AsyncLamp | Sending command {cmd:?}");
        let cmd_str = format!("{}\r\n", cmd);
        let buf: &[u8] = cmd_str.as_bytes();
        self.write(buf).await.map(|_| ()) // discard usize
    }
}

// We needed pin-project so we can access the TcpStream inside of AsyncLamp
impl AsyncRead for AsyncLamp {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        AsyncTcpStream::poll_read(self.project().stream, cx, buf)
    }
}

impl AsyncWrite for AsyncLamp {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        AsyncTcpStream::poll_write(self.project().stream, cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        self.project().stream.poll_flush(cx)
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        self.project().stream.poll_close(cx)
    }
}
