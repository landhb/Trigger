use std::mem;
use std::net::{IpAddr};
use libc::timespec;
use pnet::transport::TransportChannelType::Layer4; 
use pnet::packet::ip::IpNextHeaderProtocols;       
use pnet::transport::TransportProtocol::Ipv4;      
use pnet::transport::TransportProtocol::Ipv6;
use failure::{Error};

use crate::route;


// Drawbridge protocol data
#[repr(C,packed)]
pub struct db_data {
    timestamp: timespec,
    port: u16,
}

impl db_data {
    // db_packet method to convert to &[u8]
    // which is necessary for most libpnet methods
    pub fn as_bytes(&self) -> &[u8] {

        union Overlay<'a> {
            pkt: &'a db_data,
            bytes: &'a [u8;mem::size_of::<db_data>()],
        }
        unsafe { Overlay { pkt: self }.bytes } 
    }

}

pub struct DrawBridgePacket {
	pub db_packet_data: db_data,
	pub config: pnet::transport::TransportChannelType,
    pub proto: String,
    pub src_ip: IpAddr,
    pub target: IpAddr,
    pub dport: u16,
    pub packet_buffer: Vec<u8>,
}


impl DrawBridgePacket {
		pub fn new(proto: &String, target: IpAddr, dport: u16, unlock_port: u16, iface: String) -> Result<DrawBridgePacket, Error> {
	    // All packets will be ethernet packets
	    let mut buf_size: usize = pnet::packet::ethernet::EthernetPacket::minimum_packet_size();

	    // initialize the data
	    let mut data =  db_data {
	        port: unlock_port,
	        timestamp : libc::timespec {
	            tv_sec: 0,
	            tv_nsec:0,
	         },
	     };

	    // get current timestamp
	    unsafe {
	        libc::clock_gettime(libc::CLOCK_REALTIME,&mut data.timestamp);
	    }
		// Dynamically set the transport protocol, and calculate packet size
	    // todo, see if the header size can be calculated and returned in tcp.rs & udp.rs
		let config: pnet::transport::TransportChannelType = match (proto.as_str(),target.is_ipv4()) {
	        ("tcp",true) => {
	            buf_size += pnet::packet::ipv4::Ipv4Packet::minimum_packet_size();
	            buf_size += pnet::packet::tcp::MutableTcpPacket::minimum_packet_size();
	            Layer4(Ipv4(IpNextHeaderProtocols::Tcp))
	        },
	        ("tcp",false) => {
	            buf_size += pnet::packet::ipv6::Ipv6Packet::minimum_packet_size();
	            buf_size += pnet::packet::tcp::MutableTcpPacket::minimum_packet_size();
	            Layer4(Ipv6(IpNextHeaderProtocols::Tcp))
	        },
	        ("udp",true) => {
	            buf_size += pnet::packet::ipv4::Ipv4Packet::minimum_packet_size();
	            buf_size += pnet::packet::udp::MutableUdpPacket::minimum_packet_size();
	            Layer4(Ipv4(IpNextHeaderProtocols::Udp))
	        },
	        ("udp",false) => {
	            buf_size += pnet::packet::ipv6::Ipv6Packet::minimum_packet_size();
	            buf_size += pnet::packet::udp::MutableUdpPacket::minimum_packet_size();
	            Layer4(Ipv6(IpNextHeaderProtocols::Udp))
	        },
	        _ => unreachable!("Uhh, this shoulnd't ever happen")
	    };

	    // calculate the size of the payload
	    buf_size += mem::size_of::<db_data>();
	    // Allocate enough room for the entire packet
		let packet_buffer: Vec<u8> = vec![0;buf_size];

   	    let src_ip = route::get_interface_ip(&iface).unwrap();
 	    println!("[+] Selected Interface {}, with address {}", iface, src_ip);

 

	    return Ok(DrawBridgePacket{ db_packet_data: data,config: config, proto: proto.to_string(), src_ip: src_ip, target: target, dport: dport, packet_buffer: packet_buffer});
	}
}

