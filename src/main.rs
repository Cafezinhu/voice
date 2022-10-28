use std::{thread, net::SocketAddr};

use laminar::{Socket, Packet, SocketEvent};

// Creates the socket
fn main() {
    let mut clients: Vec<SocketAddr> = Vec::new();
    let mut socket = Socket::bind("0.0.0.0:4242").unwrap();
    let packet_sender = socket.get_packet_sender();
    let event_receiver = socket.get_event_receiver();
    let _thread = thread::spawn(move || socket.start_polling());

    println!("starting server");
    loop{
        let result = event_receiver.recv();

        match result {
            Ok(socket_event) => {
                match socket_event {
                    SocketEvent::Packet(packet) => {
                        let endpoint = packet.addr();
                        match (&clients).binary_search(&endpoint){
                            Ok(_) => {},
                            Err(_) => {
                                println!("Player connected. port: {}", endpoint.port());
                                (&mut clients).push(endpoint);
                            }
                        }
                        let received_data = packet.payload();
                        for client in &clients {
                            if client.ip() != endpoint.ip() || client.port() != endpoint.port(){
                                let unreliable = Packet::unreliable(*client, received_data.to_vec());
                                packet_sender.send(unreliable).unwrap();
                            }
                        }
                    },
                    SocketEvent::Connect(_) => {}
                    SocketEvent::Timeout(_) => {}
                    SocketEvent::Disconnect(_) => {}
                }
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
