use crate::models::server::{Leader, Server, ServerConfig, Peer, Log, LogEntry, VoteRequest, VoteResponse};
use crate::models::state::NodeState;
use std::collections::HashMap;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::net::{SocketAddrV4, Ipv4Addr};
use async_trait::async_trait;

async fn handle_socket(stream: &mut TcpStream, server: &RwLock<Server>) {
    loop {
        let mut buff = [0; 128];
        stream.read(&mut buff).await.unwrap();
        let msg: RpcMessage = bincode::deserialize(&buff).unwrap();
        eprintln!("{:?}", msg);
        let response = match msg {
            RpcMessage::Heartbeat {term, id} => {
                handle_log(&server, term, id).await
            }
            RpcMessage::VoteRequest { term, id } => {
                handle_vote_request(&server, term, id).await
            }
            _ => Vec::new()
        };

        stream.write(&response).await.unwrap();
        stream.flush().await.unwrap();
    }
}

async fn handle_log(server: &RwLock<Server>, term: usize, id: usize) -> Vec<u8> {
    let mut temp_server = server.write().await;
    
    println!("Server #{:?} with term {:?}, received a heartbeat from server #{}", temp_server.id, temp_server.term, id);
    temp_server.reset_timeout();
    if term > temp_server.term {
        println!("Server #{:?} is a new follower, following {}", temp_server.id, id);
        temp_server.term = term;
        temp_server.state = NodeState::Follower;
        temp_server.voted_for = None;
        temp_server.current_leader = Some(
            Leader {
                id,
                term 
            }
        );
        
    }
    let response = RpcMessage::HeartbeatResponse {
        term: temp_server.term,
        id: id
    };

    return bincode::serialize(&response).unwrap()
}

async fn handle_vote_request(server: &RwLock<Server>, term: usize, id: usize) -> Vec<u8> {
    let mut tmp_server = server.write().await;
    let response = match tmp_server.voted_for {
        Some(_) => {
            VoteResponse {
                term,
                response: false
            }
        }
        None => {
            if term > tmp_server.term {
                tmp_server.voted_for = Some(
                    Peer {
                        id: id,
                        address: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7879),
                    }
                );

                VoteResponse {
                    term: term,
                    response: true,
                }
            } else {
                VoteResponse {
                    term: term,
                    response: false,
                }
            }
        }
    };

    let response = RpcMessage::VoteResponse {
        term: response.term,
        response: response.response
    };

    return bincode::serialize(&response).unwrap()
}

#[async_trait]
pub trait Rpc {
    async fn request_vote(&mut self, vote: VoteRequest) -> Vec<VoteResponse>;
    async fn broadcast_log(&mut self, log: LogEntry);
}

pub struct RpcClient {
    pub servers: HashMap<usize, TcpStream>,
}

impl RpcClient {
    pub async fn new(peers: &Vec<Peer>) -> Self {
        let mut servers = HashMap::new();
        for peep in peers {
            let stream = TcpStream::connect(&peep.address).await.unwrap();
            servers.insert(peep.id, stream);
        }
        RpcClient { servers }
    }
}

#[async_trait]
impl Rpc for RpcClient {
    async fn broadcast_log(&mut self, entry: LogEntry) {
        if let LogEntry::Heartbeat { term, id } = entry {
            let rpc_msg = RpcMessage::Heartbeat {
                term,
                id
            };
            let msg = bincode::serialize(&rpc_msg).unwrap();
            for stream in self.servers.values_mut() {
                stream.write(&msg).await.unwrap();
                std::thread::sleep(std::time::Duration::from_millis(100));
                let mut buf = [0; 128];
                stream.read(&mut buf).await.unwrap();
            }
        }
    }
    async fn request_vote(&mut self, request: VoteRequest) -> Vec<VoteResponse> {
        let msg = RpcMessage::VoteRequest {
            id: request.id,
            term: request.term,
        };

        let msg_bin = bincode::serialize(&msg).unwrap();
        let mut vote_responses = Vec::new();


        for stream in self.servers.values_mut() {
            stream.write(&msg_bin).await.unwrap();
            let mut buff = [0; 128];
            stream.read(&mut buff).await.unwrap();
            let response: RpcMessage = bincode::deserialize(&buff).unwrap();
            vote_responses.push(response);
        }


        let mut responses = Vec::new();

        for vote in vote_responses {
            if let RpcMessage::VoteResponse {term, response} = vote {
                responses.push(VoteResponse {
                    term,
                    response
                });
            }
        }
        eprintln!("{:?}", responses);
        responses
    }
}

pub struct RpcServer {
    pub server: RwLock<Server>,
}

impl RpcServer {
    pub fn new(server: Server) -> Self {
        Self {
            server: RwLock::new(server)
        }
    }

    pub async fn start_server(&self) {
        let listener = TcpListener::bind(
            {
                let stream = &self.server.read().await.address;
                *stream
            }).await.unwrap();
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            handle_socket(&mut socket, &self.server).await;
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RpcMessage {
    VoteRequest { term: usize, id: usize },
    VoteResponse { term: usize, response: bool },
    Heartbeat { term: usize, id: usize },
    HeartbeatResponse { term: usize, id: usize },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LabMessage {
    Tweet { text: String, user: String, retweet_count: usize, favorite_count: usize, followers_count: usize },
    User { id: String, username: String }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    pub async fn test_handle_socket() {
        let server = Server::new(ServerConfig { timeout: std::time::Duration::from_millis(75)},1, SocketAddrV4::new(Ipv4Addr::LOCALHOST, 42069), Vec::new());
        let rpc_server = RpcServer::new(server);
        rpc_server.start_server().await;
        std::thread::sleep(std::time::Duration::from_millis(2001));
    }

    #[tokio::test]
    pub async fn test_write_socket() {
        let peers1 = vec![
            Peer {
                id: 1,
                address: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 42069)
            },
            ];
        let mut rpc_client = RpcClient::new(&peers1).await;
        let vote_request = VoteRequest { term: 1, id: 1};
        rpc_client.request_vote(vote_request).await;
    }
}
