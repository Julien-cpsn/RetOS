use crate::clock::{sleep, Clock};
use crate::terminal::error::CliError;
use crate::{println};
use alloc::collections::BTreeMap;
use alloc::vec;
use core::cmp;
use byteorder::{ByteOrder, NetworkEndian};
use goolog::{trace};
use smoltcp::iface::{SocketSet};
use smoltcp::phy::Device;
use smoltcp::socket::icmp::{Endpoint, PacketBuffer, PacketMetadata, Socket};
use smoltcp::time::{Duration, Instant};
use smoltcp::wire::{Icmpv4Packet, Icmpv4Repr, Icmpv6Packet, Icmpv6Repr, IpAddress};
use crate::devices::network::manager::NETWORK_MANAGER;

const GOOLOG_TARGET: &str = "PING";

macro_rules! send_icmp_ping {
    ($repr_type:ident, $packet_type:ident, $ident:expr, $seq_no:expr, $echo_payload:expr, $socket:expr, $remote_addr:expr) => {{
        let icmp_repr = $repr_type::EchoRequest {
            ident: $ident,
            seq_no: $seq_no,
            data: &$echo_payload,
        };

        let icmp_payload = $socket.send(icmp_repr.buffer_len(), $remote_addr).unwrap();

        let icmp_packet = $packet_type::new_unchecked(icmp_payload);
        (icmp_repr, icmp_packet)
    }};
}

macro_rules! get_icmp_pong {
    ($repr_type:ident, $repr:expr, $payload:expr, $waiting_queue:expr, $remote_addr:expr, $timestamp:expr, $received:expr) => {{
        if let $repr_type::EchoReply { seq_no, data, .. } = $repr {
            if let Some(_) = $waiting_queue.get(&seq_no) {
                let packet_timestamp_ms = NetworkEndian::read_i64(data);
                println!(
                    "{} bytes from {}: icmp_seq={}, time={}ms",
                    data.len(),
                    $remote_addr,
                    seq_no,
                    $timestamp.total_millis() - packet_timestamp_ms
                );
                $waiting_queue.remove(&seq_no);
                $received += 1;
            }
        }
    }};
}

pub fn ping(remote_addr: IpAddress) -> Result<(), CliError> {
    trace!("PING");

    let mut network_manager = NETWORK_MANAGER.lock();
    let iface_name = "eth0";
    let network_device = network_manager.interfaces.get_mut(iface_name).unwrap();
    let iface = &mut network_device.interface;
    let device_caps = network_device.network_controller.capabilities();
    let smoltcp_device = &mut network_device.network_controller;

    let icmp_rx_buffer = PacketBuffer::new(vec![PacketMetadata::EMPTY], vec![0; 256]);
    let icmp_tx_buffer = PacketBuffer::new(vec![PacketMetadata::EMPTY], vec![0; 256]);
    let icmp_socket = Socket::new(icmp_rx_buffer, icmp_tx_buffer);
    let mut sockets = SocketSet::new(vec![]);
    let icmp_handle = sockets.add(icmp_socket);

    let mut send_at = Instant::from_millis(0);
    let mut seq_no = 0;
    let mut received = 0;
    let mut echo_payload = [0xffu8; 40];
    let mut waiting_queue = BTreeMap::new();
    let ident = 0x22b;

    let count = 4;
    let interval = Duration::from_secs(1);
    let timeout = Duration::from_secs(2);

    loop {
        let timestamp = Clock::now();
        iface.poll(timestamp, smoltcp_device, &mut sockets);

        let timestamp = Clock::now();
        let socket = sockets.get_mut::<Socket>(icmp_handle);
        if !socket.is_open() {
            trace!("Binding socket");
            socket.bind(Endpoint::Ident(ident)).unwrap();
            send_at = timestamp;
            trace!("Socket bound");
        }

        if socket.can_send() && seq_no < count as u16 && send_at <= timestamp {
            NetworkEndian::write_i64(&mut echo_payload, timestamp.total_millis());

            match remote_addr {
                IpAddress::Ipv4(_) => {
                    trace!("Sending ICMP packet (IPv4)");

                    let (icmp_repr, mut icmp_packet) = send_icmp_ping!(
                        Icmpv4Repr,
                        Icmpv4Packet,
                        ident,
                        seq_no,
                        echo_payload,
                        socket,
                        remote_addr
                    );
                    icmp_repr.emit(&mut icmp_packet, &device_caps.checksum);
                }
                IpAddress::Ipv6(address) => {
                    trace!("Sending ICMP packet (IPv6)");

                    let (icmp_repr, mut icmp_packet) = send_icmp_ping!(
                        Icmpv6Repr,
                        Icmpv6Packet,
                        ident,
                        seq_no,
                        echo_payload,
                        socket,
                        remote_addr
                    );
                    icmp_repr.emit(
                        &iface.get_source_address_ipv6(&address),
                        &address,
                        &mut icmp_packet,
                        &device_caps.checksum,
                    );
                }
            }
            trace!("ICMP packet sent");

            waiting_queue.insert(seq_no, timestamp);
            seq_no += 1;
            send_at += interval;
        }

        if socket.can_recv() {
            let (payload, _) = socket.recv().unwrap();

            match remote_addr {
                IpAddress::Ipv4(_) => {
                    trace!("ICMP packet received (IPv4)");

                    let icmp_packet = Icmpv4Packet::new_checked(&payload).unwrap();
                    let icmp_repr = Icmpv4Repr::parse(&icmp_packet, &device_caps.checksum).unwrap();
                    get_icmp_pong!(
                        Icmpv4Repr,
                        icmp_repr,
                        payload,
                        waiting_queue,
                        remote_addr,
                        timestamp,
                        received
                    );
                }
                IpAddress::Ipv6(address) => {
                    trace!("ICMP packet received (IPv6)");

                    let icmp_packet = Icmpv6Packet::new_checked(&payload).unwrap();
                    let icmp_repr = Icmpv6Repr::parse(
                        &address,
                        &iface.get_source_address_ipv6(&address),
                        &icmp_packet,
                        &device_caps.checksum,
                    )
                        .unwrap();
                    get_icmp_pong!(
                        Icmpv6Repr,
                        icmp_repr,
                        payload,
                        waiting_queue,
                        remote_addr,
                        timestamp,
                        received
                    );
                }
            }
        }

        waiting_queue.retain(|seq, from| {
            if timestamp - *from < timeout {
                true
            } else {
                println!("From {remote_addr} icmp_seq={seq} timeout");
                false
            }
        });

        if seq_no == count as u16 && waiting_queue.is_empty() {
            break;
        }

        let timestamp = Clock::now();
        match iface.poll_at(timestamp, &sockets) {
            Some(poll_at) if timestamp < poll_at => {
                let resume_at = cmp::min(poll_at, send_at);
                sleep((resume_at - timestamp).secs());
            }
            Some(_) => (),
            None => {
                sleep((send_at - timestamp).secs());
            }
        }
    }

    println!("--- {remote_addr} ping statistics ---");
    println!(
        "{} packets transmitted, {} received, {:.0}% packet loss",
        seq_no,
        received,
        100.0 * (seq_no - received) as f64 / seq_no as f64
    );

    Ok(())
}