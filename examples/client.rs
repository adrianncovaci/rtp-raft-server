#[tokio::main]
async fn main() -> Result<()>{

    let mut file = File::open("msgs.txt").await?;

    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    println!("{:?}", contents);
    Ok(())
}
