use crate::models::server::{Log, VoteRequest, VoteResponse};
use std::collections::HashMap;

pub trait Rpc {
    fn request_vote(&self, vote: VoteRequest) -> Vec<VoteResponse>;
    fn broadcast_log(&self, log: Log);
}

pub struct RcpClient {
    // pub servers: HashMap<usize, TcpStream>,
}
