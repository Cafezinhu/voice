use std::{thread, net::SocketAddr, collections::HashMap, time::{SystemTime, Duration}, sync::{Arc, Mutex}};

use laminar::{Socket, Packet, SocketEvent};

struct PlayerData{
    addr: SocketAddr,
    last_time_listened: SystemTime
}
// Creates the socket
fn main() {
    let mut clients: Arc<Mutex<HashMap<String, PlayerData>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut socket = Socket::bind("0.0.0.0:4242").unwrap();
    let packet_sender = socket.get_packet_sender();
    let event_receiver = socket.get_event_receiver();
    let _thread = thread::spawn(move || socket.start_polling());

    println!("starting server");

    let clients_time_loop = Arc::clone(&clients);

    thread::spawn(move || {
        let ten_seconds = Duration::from_secs(10);
        loop {
            thread::sleep(ten_seconds);
            let mut clients = clients_time_loop.lock().unwrap();
            let mut clients_to_be_removed = Vec::new();

            for client in clients.iter() {
                if client.1.last_time_listened.elapsed().unwrap() >= ten_seconds {
                    println!("Disconnecting user {}", client.0);
                    clients_to_be_removed.push(client.0.clone());
                }
            }

            for client in clients_to_be_removed {
                clients.remove(&client);
            }
        }
    });

    let clients_addr = Arc::clone(&clients);

    loop{
        let result = event_receiver.recv();

        match result {
            Ok(socket_event) => {
                match socket_event {
                    SocketEvent::Packet(packet) => {
                        let endpoint = packet.addr();
                        let address = format!("{}:{}", endpoint.ip(), endpoint.port());
                        let mut clients = clients_addr.lock().unwrap();
                        match (&mut clients).get_mut(&address){
                            Some(client) => {
                                client.last_time_listened = SystemTime::now();
                            },
                            None => {
                                println!("Player connected: {}", address);
                                (&mut clients).insert(
                                    address, 
                                    PlayerData { 
                                        addr: endpoint, 
                                        last_time_listened: SystemTime::now()
                                    }
                                );
                            }
                        }
                        let received_data = packet.payload();
                        for client in (&clients).values() {
                            if client.addr.ip() != endpoint.ip() || client.addr.port() != endpoint.port(){
                                let unreliable = Packet::unreliable(client.addr, received_data.to_vec());
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
