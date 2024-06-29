use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    let token = env::var("DISCORD_TOKEN").expect("'DISCORD_TOKEN' environment variable not set");
    println!("{}", token);
    Ok(())
}
