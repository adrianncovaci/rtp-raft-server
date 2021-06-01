use tokio::fs::File;
use tokio::io::AsyncWriteExt; // for write_all()
use tokio::io::AsyncReadExt; // for read_to_end()


#[tokio::main]
async fn main() -> Result<(), ()>{

    let mut file = File::open("msgs.txt").await?;

    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    println!("{:?}", contents);
    Ok(())
}
