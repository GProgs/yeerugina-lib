use crate::lim::MIN_DURATION;
use log::debug;
use pin_project::pin_project;
use regex::Regex;
use smol::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use smol::net::{AsyncToSocketAddrs, TcpStream as AsyncTcpStream};

use smol::Timer;

use std::io;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::num::ParseIntError;
use std::sync::LazyLock;
use std::thread;
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

impl<'a> Lamp {
    /// Create a new Lamp from an IP address (or several addresses).
    ///
    /// The argument can be anything that implements [`ToSocketAddrs`], such as String, &str, or (&str, u16).
    /// You can pass multiple addresses into the method, and the TcpStream will use the first successful connection.
    /// If no address provides a connection, the most recent (i.e. last) error will be returned.
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
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
    pub fn connect_timeout<A: ToSocketAddrs>(addr: A, timeout: Duration) -> io::Result<Self> {
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

    // TODO: we need a send_cmd method that blocks until we get a reply from the lamp.

    /// Send a command to the lamp.
    ///
    /// This command takes a reference to a [`Command`], so it does not consume the command.
    pub fn send_cmd(&mut self, cmd: &Command) -> io::Result<()> {
        debug!("Lamp | Sending command {cmd:?}");
        let cmd_str = serde_json::to_string(cmd)?;
        write!(self, "{}\r\n", cmd_str)
    }

    /// Send a series of commands to the lamp.
    ///
    /// This command takes in anything that is an Iterator over references to Commands.
    /// Additionally, it takes a duration denoting the time between commands.
    /// Please note that dt denotes the time between SENDING commands,
    /// so it does NOT take into account the time needed to run each command.
    pub fn send_cmd_seq(
        &mut self,
        cmds: impl Iterator<Item = &'a Command>,
        dt: Duration,
    ) -> io::Result<()> {
        if dt < MIN_DURATION {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "dt must not be less than the minimum allowed effect duration".to_string(),
            ));
        }
        for cmd in cmds {
            self.send_cmd(cmd)?;
            thread::sleep(dt);
        }

        Ok(())
    }

    /// Send a command to the lamp and verify the response.
    ///
    ///
    pub fn send_and_read(&mut self, cmd: &Command) -> io::Result<String> {
        debug!("Lamp | Attempt send_and_read");

        // Implicit debug
        self.send_cmd(cmd)?;

        debug!("Lamp | Create BufReader for &self.stream");
        let mut reader = BufReader::new(&self.stream);

        debug!("Lamp | Create response buffer");
        let mut resp_buf: Vec<u8> = Vec::new();

        let resp_size = reader.read_until(b'\r', &mut resp_buf)?;
        debug!("Read {resp_size} bytes");

        let resp_str = String::from_utf8(resp_buf).map_err(Error::other)?;

        Self::verify_resp(cmd, &resp_str).map_err(Error::other)
        // Delegate verification to cmd
        //Self::verify_resp(cmd, resp_buf)
        /*
        let matching_id = if let Ok(resp) = mby_resp {
            resp.eq(&String::from("foobar"))
        } else {
            false
        };
        */
    }

    /// Verify that the response contained in resp_buf has the same ID as this command.
    fn verify_resp(cmd: &Command, resp_str: &str) -> Result<String, String> {
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"\{"id":(\d{1,3}),"result":\[(["\w\s]+)\]\}"#).unwrap());
        let caps = RE
            .captures(resp_str)
            .ok_or(String::from("String slice not matching"))?;
        let resp_id: u8 = caps
            .get(0)
            .unwrap() // guaranteed to return Some(...)
            .as_str()
            .parse()
            .map_err(|e: ParseIntError| e.to_string())?;
        if resp_id != cmd.id {
            return Err(String::from("Incorrect response ID"));
        }
        let resp_result = String::from(
            caps.get(1)
                .ok_or(String::from("No result match found"))?
                .as_str(),
        );
        Ok(resp_result)
        /*
        let id_str = cmd.id.to_string();
        let id_seq: &[u8] = id_str.as_bytes();
        let mut slider = resp_buf.windows(2);
        let idx = slider.position(|seq| seq == id_seq);
        idx.is_some()
        */
        /*
        match String::from_utf8(resp_buf) {
            Ok(resp) => true,
            Err(e) => {
                warn!("Cmd | UTF8 parse failed: {e}");
                let t = self.id.to_string().as_bytes();
                let w = resp_buf.windows(2);
                false
            }
        }
        */
    }
}

// Delegate reading/writing to the internal stream.
impl Read for Lamp {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for Lamp {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
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
    pub async fn connect<A: AsyncToSocketAddrs>(addr: A) -> io::Result<Self> {
        debug!("AsyncLamp | Attempt connect");
        let stream = AsyncTcpStream::connect(addr).await?;
        debug!("AsyncLamp | Connection Successful");
        Ok(Self { stream })
    }

    /// Send a command to the lamp using asynchronous I/O.
    ///
    /// This command takes in a reference to a [`Command`], and returns a Poll containing a Result.
    pub async fn send_cmd(&mut self, cmd: &Command) -> io::Result<()> {
        debug!("AsyncLamp | Sending command {cmd:?}");
        let cmd_str = format!("{}\r\n", serde_json::to_string(cmd)?);
        let buf: &[u8] = cmd_str.as_ref();
        self.write(buf).await.map(|_| ()) // discard usize
    }

    /// Send a series of commands to the lamp using asynchronous I/O.
    ///
    ///
    pub async fn send_cmd_seq(
        &mut self,
        cmds: impl Iterator<Item = &Command>,
        dt: Duration,
    ) -> io::Result<()> {
        if dt < MIN_DURATION {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "dt must not be less than the minimum allowed effect duration".to_string(),
            ));
        }
        for cmd in cmds {
            self.send_cmd(cmd).await?;
            let _ = Timer::after(dt).await;
        }

        Ok(())
    }
}

// We needed pin-project so we can access the TcpStream inside of AsyncLamp
impl AsyncRead for AsyncLamp {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<io::Result<usize>> {
        AsyncTcpStream::poll_read(self.project().stream, cx, buf)
    }
}

impl AsyncWrite for AsyncLamp {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        AsyncTcpStream::poll_write(self.project().stream, cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        self.project().stream.poll_flush(cx)
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        self.project().stream.poll_close(cx)
    }
}
