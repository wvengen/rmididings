use std::error::Error;
use std::net::{TcpStream, UdpSocket, TcpListener};
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::convert::TryInto;
use std::collections::HashMap;

pub extern crate rosc;

use super::super::proc::event::{Event, OscEventImpl};
use super::super::proc::EventStream;
use super::backend::{Backend, PortNum};

/// Size of the network input buffer;
const BUF_SIZE: usize = rosc::decoder::MTU;

struct OscInPort {
    udp_listener: Option<UdpSocket>,
    tcp_listener: Option<TcpListener>,
    tcp_listen_streams: Vec<TcpStream>,
}

struct OscOutPort<'a> {
    udp: bool,
    tcp: bool,
    addr: Option<&'a str>,
    tcp_connect_stream: Option<TcpStream>,
}

/// OSC Backend
pub struct OscBackend<'a> {
    in_ports: HashMap<PortNum, OscInPort>,
    out_ports: HashMap<PortNum, OscOutPort<'a>>,
    udp_sender: Option<UdpSocket>,
    buf: [u8; BUF_SIZE],
}

impl<'a> OscBackend<'a> {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            in_ports: HashMap::new(),
            out_ports: HashMap::new(),
            udp_sender: None,
            buf: [0; BUF_SIZE],
        })
    }

    fn _create_in_port(&mut self, backend_port: PortNum, name: &'a str, udp: bool, tcp: bool) -> Result<bool, Box<dyn Error>> {
        let mut udp_listener = None;
        let mut tcp_listener = None;

        if udp {
            let us = UdpSocket::bind(name)?;
            us.set_nonblocking(true)?;
            udp_listener = Some(us);
        }
        if tcp {
            let tl = TcpListener::bind(name)?;
            tl.set_nonblocking(true)?;
            tcp_listener = Some(tl);
        }

        self.in_ports.insert(backend_port, OscInPort {
            udp_listener,
            tcp_listener,
            tcp_listen_streams: vec![],
        });

        Ok(true)
    }

    fn _create_out_port(&mut self, backend_port: PortNum, _name: &'a str, udp: bool, tcp: bool) -> Result<bool, Box<dyn Error>> {
        if udp {
            if self.udp_sender.is_none() {
                self.udp_sender = Some(UdpSocket::bind("0.0.0.0:0")?);
            }
        }

        self.out_ports.insert(backend_port, OscOutPort {
            udp,
            tcp,
            addr: None,
            tcp_connect_stream: None,
        });

        Ok(true)
    }
}

impl<'a> Backend<'a> for OscBackend<'a> {
    fn set_client_name(&mut self, _name: &str) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn create_in_port(&mut self, backend_port: PortNum, name: &'a str) -> Result<bool, Box<dyn Error>> {
        if let Some((backend_name, port_name)) = name.split_once(':') {
            let port_name = port_name.strip_prefix("//").unwrap_or(port_name); // allow use of: osc://localhost:1234
            match backend_name {
                "osc" => self._create_in_port(backend_port, port_name, true, true),
                "osc.udp" => self._create_in_port(backend_port, port_name, true, false),
                "osc.tcp" => self._create_in_port(backend_port, port_name, false, true),
                _ => Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn create_out_port(&mut self, backend_port: PortNum, name: &'a str) -> Result<bool, Box<dyn Error>> {
        if let Some((backend_name, port_name)) = name.split_once(':') {
            let port_name = port_name.strip_prefix("//").unwrap_or(port_name); // allow use of: osc://localhost:1234
            match backend_name {
                "osc" => self._create_out_port(backend_port, port_name, true, true),
                "osc.udp" => self._create_out_port(backend_port, port_name, true, false),
                "osc.tcp" => self._create_out_port(backend_port, port_name, false, true),
                _ => Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn connect_in_port(&mut self, _backend_port: PortNum, _name: &'a str) -> Result<bool, Box<dyn Error>> {
        // Not applicable, others connect to our in ports.
        Ok(false)
    }

    fn connect_out_port(&mut self, backend_port: PortNum, name: &'a str) -> Result<bool, Box<dyn Error>> {
        if let Some(port) = self.out_ports.get_mut(&backend_port) {
            port.addr = Some(name);

            // UDP needs no connection setup, we just send it.

            if port.tcp {
                if let Ok(stream) = TcpStream::connect(name) {
                    stream.set_nonblocking(true)?;
                    port.tcp_connect_stream = Some(stream);
                    println!("OSC connection to {} succeeded.", name);
                } else {
                    // TODO better warning system
                    // TODO allow connecting later (requires pollfds update during run)
                    println!("OSC connection to {} failed.", name);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_pollfds(&mut self) -> Result<Vec<libc::pollfd>, Box<dyn Error>> {
        let mut pollfds: Vec<libc::pollfd> = vec![];

        for port in self.in_ports.values() {
            if let Some(udp_listener) = &port.udp_listener {
                pollfds.push(libc::pollfd { fd: udp_listener.as_raw_fd(), events: 1, revents: 0 });
            }
            if let Some(tcp_listener) = &port.tcp_listener {
                pollfds.push(libc::pollfd { fd: tcp_listener.as_raw_fd(), events: 1, revents: 0 });
            }
            // TODO This doesn't work, as there are no pollfds yet, only after connecting
            //      and they are not picked up. Needs pollfds update during run.
            for tcp_stream in port.tcp_listen_streams.iter() {
                pollfds.push(libc::pollfd { fd: tcp_stream.as_raw_fd(), events: 1, revents: 0 });
            }
        }

        Ok(pollfds)
    }

    fn run<'evs: 'run, 'run>(&'run mut self) -> Result<EventStream<'evs>, Box<dyn Error>> {
        let mut evs = EventStream::empty();

        for port in self.in_ports.values_mut() {
            if let Some(udp_listener) = &port.udp_listener {
                if let Some(data) = read_udp_data(&udp_listener, &mut self.buf)? {
                    evs.extend(decode_data(data).into_iter().map(|o| Event::from(o)));
                }
            }

            if let Some(tcp_listener) = &port.tcp_listener {
                // TODO move to function
                for stream in tcp_listener.incoming() {
                    match stream {
                        Ok(stream) => {
                            stream.set_nonblocking(true)?;
                            // stream.set_nodelay(true)?;
                            port.tcp_listen_streams.push(stream);
                        },
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => { break; },
                        Err(e) => { return Err(Box::new(e)) },
                    }
                }
            }

            for tcp_stream in port.tcp_listen_streams.iter_mut() {
                if let Some(data) = read_tcp_data(tcp_stream, &mut self.buf)? {
                    evs.extend(decode_data_tcp(data).into_iter().map(|o| Event::from(o)));
                }
            }
        }

        // This isn't really used in practice, as OSC doesn't keep open connections AFAIK.
        for port in self.out_ports.values_mut() {
            if let Some(tcp_stream) = &mut port.tcp_connect_stream {
                if let Some(data) = read_tcp_data(tcp_stream, &mut self.buf)? {
                    evs.extend(decode_data_tcp(data).into_iter().map(|o| Event::from(o)));
                }
            }
        }

        Ok(evs)
    }

    fn output_event(&mut self, ev: &Event) -> Result<u32, Box<dyn Error>> {
        match ev {
            Event::Osc(ref ev) => { self._output_event(ev) },
            _ => Ok(0)
        }
    }
}

impl<'a> OscBackend<'a> {
    fn _output_event(&mut self, ev: &OscEventImpl) -> Result <u32, Box<dyn Error>> {
        let mut bytes = 0;

        // Use indicated port, but if there is only one OSC port, use that for ease of use.
        let mut backend_port = ev.port;
        if self.out_ports.len() == 1 {
            if let Some(p) = self.out_ports.keys().next() {
                backend_port = *p;
            }
        }

        if let Some(port) = self.out_ports.get_mut(&backend_port) {
            if port.udp {
                if let Some(addr) = &port.addr {
                    if let Some(socket) = &self.udp_sender {
                        bytes += send_osc_udp(socket, addr, &ev.addr, &ev.args)?;
                    }
                }
            }
            if port.tcp {
                if let Some(_) = &port.tcp_connect_stream {
                    // We already have a stream, nothing to do.
                } else if let Some(addr) = &port.addr {
                    if let Ok(stream) = TcpStream::connect(addr) {
                        stream.set_nonblocking(true)?;
                        port.tcp_connect_stream = Some(stream);
                        println!("OSC connection to {} succeeded, will retry later.", addr);
                    }
                }

                if let Some(tcp_stream) = &mut port.tcp_connect_stream {
                    bytes += send_osc_tcp(tcp_stream, &ev.addr, &ev.args)?;
                }
            }
        }

        Ok(bytes as u32)
    }

}

fn send_osc_udp(socket: &UdpSocket, dest: &str, addr: &str, args: &Vec<rosc::OscType>) -> Result<usize, Box<dyn Error>> {
    let message = rosc::OscMessage { addr: String::from(addr), args: args.clone() };
    let data = rosc::encoder::encode(&rosc::OscPacket::Message(message))?;
    Ok(socket.send_to(&data, &dest)?)
}

fn send_osc_tcp(stream: &mut TcpStream, addr: &str, args: &Vec<rosc::OscType>) -> Result<usize, Box<dyn Error>> {
    let message = rosc::OscMessage { addr: String::from(addr), args: args.clone() };
    let data = rosc::encoder::encode(&rosc::OscPacket::Message(message))?;
    // https://github.com/klingtnet/rosc/issues/19
    let mut bytes = 0;
    bytes += stream.write(&(data.len() as i32).to_be_bytes())?;
    bytes += stream.write(&data)?;
    stream.flush()?;
    Ok(bytes)
}

fn read_udp_data<'a>(socket: &UdpSocket, data: &'a mut [u8]) -> Result<Option<&'a [u8]>, Box<dyn Error>> {
    match socket.recv_from(data) {
        Ok((n, _addr)) => Ok(Some(&data[..n])),
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(Box::new(e)),
    }
}

fn decode_data(data: &[u8]) -> Vec::<rosc::OscMessage> {
    if let Ok(packet) = rosc::decoder::decode(data) {
        get_messages_from_packet(packet)
    } else {
        // silently ignore malformed packets
        vec![]
    }
}

fn read_tcp_data<'a>(stream: &mut TcpStream, data: &'a mut [u8]) -> Result<Option<&'a [u8]>, Box<dyn Error>> {
    match stream.read(data) {
        Ok(n) => Ok(Some(&data[..n])),
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(Box::new(e)),
    }
}

fn decode_data_tcp(data: &[u8]) -> Vec::<rosc::OscMessage> {
    // https://github.com/klingtnet/rosc/issues/19
    let mut messages = Vec::<rosc::OscMessage>::new();

    let mut i: usize = 0;
    while i < data.len() {
        if let Ok(packet_len_bytes) = data[i..i+4].try_into() {
            let packet_len = i32::from_be_bytes(packet_len_bytes) as usize;
            if packet_len > data.len() - 4 - i { break; }
            messages.extend(decode_data(&data[i+4..i+4+packet_len]));
            i += 4 + packet_len;
        } else {
            break;
        }
    }

    messages
}

fn get_messages_from_packet(packet: rosc::OscPacket) -> Vec::<rosc::OscMessage> {
    match packet {
        rosc::OscPacket::Message(msg) => {
            vec!(msg)
        },
        rosc::OscPacket::Bundle(bundle) => {
            bundle.content.into_iter().map(|p| get_messages_from_packet(p)).flatten().collect()
        },
    }
}