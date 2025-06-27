mod ais;
mod buddy;
mod error;
mod utils;

use std::io::{self, Write};
use textwrap::wrap;
use tracing_subscriber::{EnvFilter, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{ buddy::Buddy};

pub use self::error::{Error, Result};

const DEFAULT_DIR: &str = "buddy";

enum Cmd {
    Quit,
    Chat(String),
    RefreshAll,
    RefreshConv,
    RefreshInst,
    RefreshFiles,
}

impl Cmd {
    fn from_input(input: impl Into<String>) -> Self {
        let input = input.into();
        match input.as_str() {
            "/q" => Self::Quit,
            "/r" | "/ra" => Self::RefreshAll,
            "/ri" => Self::RefreshInst,
            "/rf" => Self::RefreshFiles,
            "/rc" => Self::RefreshConv,
            _ => Self::Chat(input),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::registry();
    subscriber
        .with(EnvFilter::from_default_env())
        .with(Layer::default())
        .try_init()?;

    match start().await {
        Ok(_) => tracing::info!("\nBye!\n"),
        Err(e) => tracing::error!("Error: {e}"),
    }

    Ok(())
}

async fn start() -> Result<()> {
    let buddy = Buddy::init_from_dir(DEFAULT_DIR, false, false).await?;

    let conv = buddy.load_or_create_conv(false).await?;

    loop {
        println!();
        print!("Ask away: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();
        let cmd = Cmd::from_input(input);

        match cmd {
            Cmd::Quit => break,
            Cmd::Chat(msg) => {
                let res = buddy.chat(&conv, &msg).await?;
                let res = wrap(&res, 80).join("\n");
                println!("{res}");
            }
            _ => {}
        }
    }

    Ok(())
}
