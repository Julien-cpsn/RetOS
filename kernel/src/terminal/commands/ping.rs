use alloc::borrow::ToOwned;
use crate::clock::{sleep, Clock};
use crate::terminal::error::CliError;
use crate::{println};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use byteorder::{ByteOrder, NetworkEndian};
use goolog::{trace};
use no_std_clap_macros::Args;
use smoltcp::phy::Device;
use smoltcp::socket::icmp::{Endpoint, PacketBuffer, PacketMetadata, Socket};
use smoltcp::time::{Duration, Instant};
use smoltcp::wire::{Icmpv4Packet, Icmpv4Repr, Icmpv6Packet, Icmpv6Repr, IpAddress};
use crate::devices::network::manager::NETWORK_MANAGER;
use crate::terminal::custom_arguments::ip_address::IpAddressArg;

const GOOLOG_TARGET: &str = "PING";

#[derive(Args)]
pub struct PingCommand {
    /// IP address to ping
    pub ip_address: IpAddressArg,

    /// Ping count
    #[arg(default_value = "4")]
    pub count: u16,

    /// Timeout
    #[arg(default_value = "2")]
    pub timeout: u64
}

pub fn ping(remote_addr: IpAddress, count: u16, timeout: u64) -> Result<(), CliError> {
    trace!("PING");

    if remote_addr.is_unspecified() {
        return Err(CliError::Message(String::from("The given address is not addressable")));
    }

    if !remote_addr.is_unicast() {
        return Err(CliError::Message(String::from("The given address is not unicast")));
    }

    // TODO
    let iface_name = "eth0";

    let mut manager = NETWORK_MANAGER.lock();
    let local_device = manager.interfaces.get_mut(iface_name).unwrap().clone();
    drop(manager);

    let device_caps = local_device.lock().network_controller.capabilities();
    let local_sockets = local_device.lock().sockets.clone();

    let icmp_rx_buffer = PacketBuffer::new(vec![PacketMetadata::EMPTY], vec![0; 256]);
    let icmp_tx_buffer = PacketBuffer::new(vec![PacketMetadata::EMPTY], vec![0; 256]);
    let icmp_socket = Socket::new(icmp_rx_buffer, icmp_tx_buffer);

    let icmp_handle = local_sockets.lock().add(icmp_socket);

    let mut send_at = Instant::from_millis(0);
    let mut seq_no = 0;
    let mut received = 0;
    let mut echo_payload = [0xffu8; 40];
    let mut waiting_queue = BTreeMap::new();
    let ident = 0x22b;

    let interval = Duration::from_secs(1);
    let timeout = Duration::from_secs(timeout);

    loop {
        let timestamp = Clock::now();
        let (can_send, can_receive) = {
            let mut sockets = local_sockets.lock();
            let socket = sockets.get_mut::<Socket>(icmp_handle);

            if !socket.is_open() {
                trace!("Binding socket");
                socket.bind(Endpoint::Ident(ident)).unwrap();
                send_at = timestamp;
                trace!("Socket bound");
            }

            (socket.can_send(), socket.can_recv())
        };

        if can_send && seq_no < count && send_at <= timestamp {
            NetworkEndian::write_i64(&mut echo_payload, timestamp.total_millis());

            match remote_addr {
                IpAddress::Ipv4(_) => {
                    trace!("Sending ICMP packet (IPv4)");

                    let mut sockets = local_sockets.lock();
                    let socket = sockets.get_mut::<Socket>(icmp_handle);

                    let icmp_repr = Icmpv4Repr::EchoRequest {
                        ident,
                        seq_no,
                        data: &echo_payload,
                    };

                    let icmp_payload = socket.send(icmp_repr.buffer_len(), remote_addr).unwrap();
                    let mut icmp_packet = Icmpv4Packet::new_unchecked(icmp_payload);

                    icmp_repr.emit(&mut icmp_packet, &device_caps.checksum);
                }
                IpAddress::Ipv6(address) => {
                    trace!("Sending ICMP packet (IPv6)");

                    let mut sockets = local_sockets.lock();
                    let socket = sockets.get_mut::<Socket>(icmp_handle);

                    let icmp_repr = Icmpv6Repr::EchoRequest {
                        ident,
                        seq_no,
                        data: &echo_payload,
                    };

                    let icmp_payload = socket.send(icmp_repr.buffer_len(), remote_addr).unwrap();
                    let mut icmp_packet = Icmpv6Packet::new_unchecked(icmp_payload);

                    icmp_repr.emit(
                        &local_device.lock().interface.get_source_address_ipv6(&address),
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

        if can_receive {
            let payload = {
                let mut sockets = local_sockets.lock();
                let socket = sockets.get_mut::<Socket>(icmp_handle);

                let (payload, _) =socket.recv().unwrap();
                payload.to_owned()
            };

            match remote_addr {
                IpAddress::Ipv4(_) => {
                    trace!("ICMP packet received (IPv4)");

                    let icmp_packet = Icmpv4Packet::new_checked(&payload).unwrap();
                    let icmp_repr = Icmpv4Repr::parse(&icmp_packet, &device_caps.checksum).unwrap();

                    if let Icmpv4Repr::EchoReply { seq_no, data, .. } = icmp_repr {
                        handle_reply(
                            &mut waiting_queue,
                            seq_no,
                            data,
                            remote_addr,
                            timestamp,
                            &mut received
                        );
                    }
                }
                IpAddress::Ipv6(address) => {
                    trace!("ICMP packet received (IPv6)");

                    let icmp_packet = Icmpv6Packet::new_checked(&payload).unwrap();
                    let icmp_repr = Icmpv6Repr::parse(
                        &address,
                        &local_device.lock().interface.get_source_address_ipv6(&address),
                        &icmp_packet,
                        &device_caps.checksum,
                    )
                        .unwrap();

                    if let Icmpv6Repr::EchoReply { seq_no, data, .. } = icmp_repr {
                        handle_reply(
                            &mut waiting_queue,
                            seq_no,
                            data,
                            remote_addr,
                            timestamp,
                            &mut received
                        );
                    }
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

        if seq_no >= count && waiting_queue.is_empty() {
            break;
        }

        trace!("Loop");
        sleep(1);
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

fn handle_reply(waiting_queue: &mut BTreeMap<u16, Instant>, seq_no: u16, data: &[u8], remote_addr: IpAddress, timestamp: Instant, received: &mut u16) {
    if waiting_queue.get(&seq_no).is_some() {
        let packet_timestamp_ms = NetworkEndian::read_i64(data);

        println!(
            "{} bytes from {}: icmp_seq={}, time={}ms",
            data.len(),
            remote_addr,
            seq_no,
            timestamp.total_millis() - packet_timestamp_ms
        );

        waiting_queue.remove(&seq_no);
        *received += 1;
    }
}
