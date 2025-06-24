use llm_chain::traits::Executor;
use llm_chain::{executor, parameters, prompt};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let executor = executor!()?;
    let res = prompt!(
        "You are a robot assistant for making personalized greetings",
        "Make a personalized greeting for Joe"
    )
    .run(&parameters!(), &executor)
    .await?;

    println!("{res}");
    Ok(())
}
