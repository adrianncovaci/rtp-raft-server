use std::{
    net::SocketAddrV4,
    time::{Duration, Instant},
};
use serde::{Deserialize, Serialize};

use super::state::NodeState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResponse {
    pub term: usize,
    pub response: bool,
}

#[derive(Debug, Clone)]
pub struct VoteRequest {
    pub id: usize,
    pub term: usize,
}
pub struct Log {
    pub entries: Vec<LogEntry>,
}

#[derive(Debug, Clone)]
pub struct Leader {
    pub id: usize,
    pub term: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum LogEntry {
    Heartbeat { term: usize, id: usize } 
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub id: usize,
    pub term: usize,
    pub address: SocketAddrV4,
    pub peers: Vec<Peer>,
    pub state: NodeState,
    pub votes: usize,
    pub voted_for: Option<Peer>,
    pub config: ServerConfig,
    pub timeout: Option<Instant>,
    pub current_leader: Option<Leader>,
}

impl Server {
    pub fn new(config: ServerConfig, id: usize, address: SocketAddrV4) -> Self {
        Self {
            id,
            term: 0,
            address,
            peers: Vec::new(),
            state: NodeState::Follower,
            votes: 0,
            voted_for: None,
            config,
            timeout: None,
            current_leader: None
        }
    }

    pub fn promote_leader(&mut self) {
        match self.state {
            NodeState::Candidate => {
                self.state = NodeState::Leader;
                self.term += 1;
                self.timeout = None;
            }
            _ => (),
        }
    }

    pub fn reset_timeout(&mut self) {
        self.timeout = Some(Instant::now() + self.config.timeout);
    }

    pub fn start(&mut self) {
        self.reset_timeout();
    }

    pub fn has_timed_out(&self) -> bool {
        match self.timeout {
            Some(time) => Instant::now() > time,
            None => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Peer {
    pub id: usize,
    pub address: SocketAddrV4,
}

mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use super::*;
    #[test]
    pub fn test_new_server() {
        let mut server1 = Server::new(
            ServerConfig {
                timeout: Duration::from_millis(50),
            },
            1,
            SocketAddrV4::new(Ipv4Addr::LOCALHOST, 42069),
            Vec::new(),
        );

        server1.start();
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(server1.has_timed_out(), true);
    }

    #[test]
    pub fn promote_to_leader() {
        let mut server1 = Server::new(
            ServerConfig {
                timeout: Duration::from_millis(50),
            },
            1,
            SocketAddrV4::new(Ipv4Addr::LOCALHOST, 42069),
            Vec::new(),
        );
        server1.state = NodeState::Candidate;
        server1.promote_leader();

        assert_eq!(server1.state, NodeState::Leader);
    }
}
