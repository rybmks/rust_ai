use std::time::Instant;

use kalosm::{language::*, *};
type Error = Box<dyn std::error::Error>;
type Result<T> = core::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut llm = Llama::phi_3().await.unwrap();

    let prompt = "The following is a 300 word essay about beer:";
    print!("{prompt}");

    let now = Instant::now();
    let mut stream = llm(prompt);

    stream.to_std_out().await.unwrap();
    println!("{:?}", now.elapsed());
    Ok(())
}
