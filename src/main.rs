mod models;
mod raft;
#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    let derp = 2;
}
