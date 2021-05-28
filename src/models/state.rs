#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeState {
    Follower,
    Candidate,
    Leader,
}
