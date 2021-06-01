pub mod models;
pub mod raft;
pub mod utils;
use std::{io::Error, net::{Ipv4Addr, SocketAddrV4}, time::Duration};
use crate::utils::*;
use models::server::{Peer, Server, ServerConfig};
use raft::rpc::{Rpc, RpcClient, RpcServer};
use tokio::sync::RwLock;

async fn start_server(server: RwLock<Server>, rpc_client: &mut impl Rpc) {
    server.write().await.start();
    loop {
        handle_timeout(&server, rpc_client).await;
        broadcast_heartbeat(&server, rpc_client).await;
        process_tweets(&server).await;
    }
}



#[tokio::main]
async fn main() -> Result<(), Error>{
    let mut rpc_servers = Vec::new();

    let address_1 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3300);
    let server_1 = Server::new(
        ServerConfig {
            timeout: Duration::from_millis(100),
        },
        2,
        address_1,
    );
    let address_1_peers = vec![
        Peer {
            id: 2,
            address: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3301),
        },
        Peer {
            id: 3,
            address: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3302),
        },
    ];

    rpc_servers.push(RpcServer::new(server_1.clone()));

    let address_2 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3301);
    let server_2 = Server::new(
        ServerConfig {
            timeout: Duration::from_millis(100),
        },
        2,
        address_2,
    );
    let address_2_peers = vec![
        Peer {
            id:1, 
            address: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3300),
        },
        Peer {
            id: 3,
            address: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3302),
        },
    ];

    rpc_servers.push(RpcServer::new(server_2.clone()));

    let address_3 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3302);
    let server_3 = Server::new(
        ServerConfig {
            timeout: Duration::from_millis(100),
        },
        2,
        address_3,
    );
    let address_3_peers = vec![
        Peer {
            id: 1,
            address: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3300),
        },
        Peer {
            id: 2,
            address: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3301),
        },
    ];

    rpc_servers.push(RpcServer::new(server_3.clone()));

    let server_join_handlers: Vec<_> = rpc_servers.into_iter()
        .map(|server| {
            let handle = tokio::runtime::Handle::current();

            std::thread::spawn(move || {

                    handle.spawn(async move {
                        server.start_server().await;
                    });
            })
        })
        .collect();

    for handle in server_join_handlers {
        handle.join().expect("Thread panicked");
    }

    std::thread::sleep(Duration::from_millis(1000));

    let mut client = RpcClient::new(&address_1_peers).await;

    start_server(RwLock::new(server_1), &mut client).await;


    let mut client2 = RpcClient::new(&address_2_peers).await;

    start_server(RwLock::new(server_2), &mut client2).await;


    let mut client3 = RpcClient::new(&address_3_peers).await;

    start_server(RwLock::new(server_3), &mut client3).await;




    Ok(())
}
