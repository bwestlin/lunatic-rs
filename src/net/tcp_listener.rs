use std::io::{Error, ErrorKind, Result};
use std::net::SocketAddr;

use super::{SocketAddrIterator, TcpStream};
use crate::{error::LunaticError, host_api};

/// A TCP server, listening for connections.
///
/// After creating a [`TcpListener`] by [`bind`][`TcpListener::bind()`]ing it to an address, it
/// listens for incoming TCP connections. These can be accepted by calling
/// [`accept()`][`TcpListener::accept()`].
///
/// The Transmission Control Protocol is specified in [IETF RFC 793].
///
/// [IETF RFC 793]: https://tools.ietf.org/html/rfc793
///
/// # Examples
///
/// ```no_run
/// use lunatic::{net, process, Mailbox};
/// use std::io::{BufRead, BufReader, Write};
///
/// fn main() {
///     let listener = net::TcpListener::bind("127.0.0.1:1337").unwrap();
///     while let Ok((tcp_stream, _peer)) = listener.accept() {
///         // Handle connections in a new process
///         process::spawn_with(tcp_stream, handle).unwrap();
///     }
/// }
///
/// fn handle(mut tcp_stream: net::TcpStream, _: Mailbox<()>) {
///     let mut buf_reader = BufReader::new(tcp_stream.clone());
///     loop {
///         let mut buffer = String::new();
///         let read = buf_reader.read_line(&mut buffer).unwrap();
///         if buffer.contains("exit") || read == 0 {
///             return;
///         }
///         tcp_stream.write(buffer.as_bytes()).unwrap();
///     }
/// }
/// ```
#[derive(Debug)]
pub struct TcpListener {
    id: u64,
}

impl Drop for TcpListener {
    fn drop(&mut self) {
        unsafe { host_api::networking::drop_tcp_listener(self.id) };
    }
}

impl TcpListener {
    /// Creates a new [`TcpListener`] bound to the given address.
    ///
    /// Binding with a port number of 0 will request that the operating system assigns an available
    /// port to this listener.
    ///
    /// If `addr` yields multiple addresses, binding will be attempted with each of the addresses
    /// until one succeeds and returns the listener. If none of the addresses succeed in creating a
    /// listener, the error from the last attempt is returned.
    pub fn bind<A>(addr: A) -> Result<Self>
    where
        A: super::ToSocketAddrs,
    {
        let mut id = 0;
        for addr in addr.to_socket_addrs()? {
            let result = match addr {
                SocketAddr::V4(v4_addr) => {
                    let ip = v4_addr.ip().octets();
                    let port = v4_addr.port() as u32;
                    unsafe {
                        host_api::networking::tcp_bind(
                            4,
                            ip.as_ptr(),
                            port,
                            0,
                            0,
                            &mut id as *mut u64,
                        )
                    }
                }
                SocketAddr::V6(v6_addr) => {
                    let ip = v6_addr.ip().octets();
                    let port = v6_addr.port() as u32;
                    let flow_info = v6_addr.flowinfo();
                    let scope_id = v6_addr.scope_id();
                    unsafe {
                        host_api::networking::tcp_bind(
                            6,
                            ip.as_ptr(),
                            port,
                            flow_info,
                            scope_id,
                            &mut id as *mut u64,
                        )
                    }
                }
            };
            if result == 0 {
                return Ok(Self { id });
            }
        }
        let lunatic_error = LunaticError::from(id);
        Err(Error::new(ErrorKind::Other, lunatic_error))
    }

    /// Accepts a new incoming connection.
    ///
    /// Returns a TCP stream and the peer address in forma of an iterator containing only 1 element.
    pub fn accept(&self) -> Result<(TcpStream, SocketAddrIterator)> {
        let mut tcp_stream_or_error_id = 0;
        let mut dns_iter_id = 0;
        let result = unsafe {
            host_api::networking::tcp_accept(
                self.id,
                &mut tcp_stream_or_error_id as *mut u64,
                &mut dns_iter_id as *mut u64,
            )
        };
        if result == 0 {
            let tcp_stream = TcpStream::from(tcp_stream_or_error_id);
            let dns_iter = SocketAddrIterator::from(dns_iter_id);
            Ok((tcp_stream, dns_iter))
        } else {
            let lunatic_error = LunaticError::from(tcp_stream_or_error_id);
            Err(Error::new(ErrorKind::Other, lunatic_error))
        }
    }
}
