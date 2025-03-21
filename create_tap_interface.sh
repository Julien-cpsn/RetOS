sudo brctl addbr br0
sudo ip addr add 192.168.179.1/24 broadcast 192.168.179.255 dev br0
sudo ip tuntap add dev tap0 mode tap
sudo ip link set tap0 up promisc on
sudo brctl addif br0 tap0
sudo dnsmasq --interface=br0 --bind-interfaces --dhcp-range=192.168.179.10,192.168.179.254