import socket
import time
import struct

def create_network_packet():
    # Adresse MAC source (6 octets)
    src_mac = bytearray((0xc0, 0x47, 0x0e, 0x6b, 0x84, 0x79))
    # Adresse MAC destination (6 octets)
    dst_mac = bytearray((0x52, 0x54, 0x0, 0x12, 0x34, 0x56))
    # Type Ethernet (IPv4 : 0x0800)
    eth_type = b'\x08\x00'

    # Version et IHL (IPv4, 5 mots de 32 bits)
    version_ihl = b'\x45'
    # Type de service
    dscp_ecn = b'\x00'
    # Longueur totale (20 octets IP + 8 octets UDP + 4 octets payload)
    total_length = struct.pack('!H', 20 + 8 + 4 + 128)
    # Identification
    identification = b'\x00\x01'
    # Flags et fragmentation
    flags_fragment_offset = b'\x40\x00'
    # TTL
    ttl = b'\x40'
    # Protocole (UDP : 0x11)
    protocol = b'\x11'
    # Checksum (mis à 0 pour simplifier)
    header_checksum = b'\x00\x00'
    # Adresse IP source (192.168.1.1)
    src_ip = b'\xc0\xa8\x01\x01'
    # Adresse IP destination (192.168.1.2)
    dst_ip = bytearray((192, 168, 179, 1))

    # Port source (12345)
    src_port = struct.pack('!H', 12345)
    # Port destination (80)
    dst_port = struct.pack('!H', 80)
    # Longueur du segment UDP (8 + 4)
    udp_length = struct.pack('!H', 8 + 4)
    # Checksum UDP (mis à 0 pour simplifier)
    udp_checksum = b'\x00\x00'

    # Données (4 octets, "PING")
    payload = b'PING' + (b'\x00' * 128)

    # Assemblage du paquet
    ethernet_header = dst_mac + src_mac + eth_type
    ip_header = (version_ihl + dscp_ecn + total_length + identification +
                 flags_fragment_offset + ttl + protocol + header_checksum +
                 src_ip + dst_ip)
    udp_header = src_port + dst_port + udp_length + udp_checksum

    packet = ethernet_header + ip_header + udp_header + payload
    return bytearray(packet)

s = socket.socket(socket.AF_PACKET, socket.SOCK_RAW)
s.bind(("tap0", 0))
ping_frame = create_network_packet()

while True:
    print("ping")
    s.send(bytearray(ping_frame))
    time.sleep(1)