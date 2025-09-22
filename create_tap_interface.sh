sudo brctl addbr br0
sudo ip addr add 192.168.179.1/24 broadcast 192.168.179.255 dev br0

#sudo brctl addbr br1
#sudo ip addr add 192.168.179.2/24 broadcast 192.168.179.255 dev br1

sudo ip tuntap add dev tap0 mode tap
sudo ip link set tap0 up promisc on
sudo brctl addif br0 tap0

sudo ip tuntap add dev tap1 mode tap
sudo ip link set tap1 up promisc on
sudo brctl addif br0 tap1

sudo dnsmasq --interface=br0 --bind-interfaces --dhcp-range=192.168.179.10,192.168.179.254
sudo ip link set br0 up

#sudo dnsmasq --interface=br1 --bind-interfaces --dhcp-range=192.168.179.11,192.168.179.254
#sudo ip link set br1 up