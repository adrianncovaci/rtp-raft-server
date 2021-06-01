use tokio::{fs::File, io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener, sync::RwLock};

use crate::{models::{server::{LogEntry, Peer, Server, VoteRequest, VoteResponse}, state::NodeState}, raft::rpc::{LabMessage, Rpc}};

pub async fn broadcast_heartbeat(server: &RwLock<Server>, rpc_client: &mut impl Rpc) {
    let is_leader = server.read().await.state == NodeState::Leader;
    if is_leader {
        let term = server.read().await.term;
        let id = server.read().await.id;

        rpc_client.broadcast_log(LogEntry::Heartbeat {
            term,
            id,
        }).await;
    }
}

pub async fn handle_timeout(server: &RwLock<Server>, rpc_client: &mut impl Rpc) {
    let id = server.read().await.id;
    let timed_out = server.read().await.has_timed_out();

    if timed_out {
        println!("Server {} has timed out.", id);

        new_election(server, rpc_client).await;
    }
}

async fn new_election(server: &RwLock<Server>, rpc_client: &mut impl Rpc) {
    let vote_request = prepare_vote_request(&server).await;
    let server_id = server.read().await.id;
    let server_current_term = server.read().await.term;

    println!("Server {}, with term {}, started the election process.", server_id, server_current_term);

    let vote_response = match vote_request {
        Some(request) => Some(rpc_client.request_vote(request).await),
        None => None,
    };

    if let Some(r) = vote_response {
        let own_election;
        {
            let server = server.read().await;
            own_election = has_won_the_election(&server, r) && !server.has_timed_out();
        }

        if own_election {
            become_leader(&server, rpc_client).await;
        }
    }
}

async fn prepare_vote_request(server: &RwLock<Server>) -> Option<VoteRequest> {
    if server.read().await.state == NodeState::Leader {
        return None;
    }

    {
        let mut server_tmp = server.write().await;
        server_tmp.state = NodeState::Candidate;
        server_tmp.term = server_tmp.term + 1;
        server_tmp.reset_timeout();
        server_tmp.voted_for = Some(Peer {
            id: server_tmp.id,
            address: server_tmp.address,
        });
    }

    let new_term = server.read().await.term;
    let id = server.read().await.id;

    Some(VoteRequest {
        term: new_term,
        id,
    })
}

fn has_won_the_election(server: &Server, response: Vec<VoteResponse>) -> bool {
    let number_of_servers = server.peers.len() + 1; // All peers + current server

    let votes = response.iter().filter(|r| r.response).count();

    let min_quorum = ((number_of_servers / 2) as f64).floor();

    (votes + 1) > min_quorum as usize && NodeState::Candidate == server.state
}

async fn become_leader(server: &RwLock<Server>, rpc_client: &mut impl Rpc) {
    let mut server = server.write().await;

    server.promote_leader();

    let log_entry = LogEntry::Heartbeat {
        term: server.term,
        id: server.id,
    };

    rpc_client.broadcast_log(log_entry).await;
}

pub async fn process_tweets(server: &RwLock<Server>) {
    let is_leader = server.read().await.state == NodeState::Leader;

    if is_leader {
        let listener = TcpListener::bind("127.0.0.1:42069").await.unwrap();

        let (mut sock, _) = listener.accept().await.unwrap();
        let mut buff = [0; 1024];
        sock.read(&mut buff).await.unwrap();
        let mut file = File::create("msgs.txt").await.unwrap();
        file.write_all(&buff).await.unwrap();
    }
}
