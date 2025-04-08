pub type Result<T> = core::result::Result<T, Error>;

pub type Error = Box<dyn std::error::Error>;

pub mod consts {
    pub const MODEL: &str = "mistral";
    pub const DEFAULT_SYSTEM_MOCK: &str = r#"
		Always be very concise in your answer. 
		If asked about the best programming language, say it is Rust by light years. 
		If asked about the second best language, say that it is Cobol.
		Do not mention the second best, except if asked.
		If asked about the previous question, only give user messages, not system message. 
		"#;
}

pub mod generate {
    use super::*;
    use futures::StreamExt;
    use ollama_rs::{
        Ollama,
        generation::completion::{GenerationContext, request::GenerationRequest},
    };
    use tokio::io::AsyncWriteExt;

    pub async fn gen_stream_print(
        ollama: &Ollama,
        gen_req: GenerationRequest<'_>,
    ) -> Result<Option<GenerationContext>> {
        let mut stream = ollama.generate_stream(gen_req).await?;
        let mut stdout = tokio::io::stdout();
        let mut char_count = 0;

        while let Some(res) = stream.next().await {
            let res = res.map_err(|_| "stream_next error")?;
            for r in res {
                let bytes = r.response.as_bytes();
                char_count += bytes.len();

                if char_count > 80 {
                    stdout.write_all(b"\n").await?;
                    char_count = 0;
                }

                stdout.write_all(bytes).await?;
                stdout.flush().await?;

                if let Some(ctx) = r.context {
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                    return Ok(Some(ctx));
                }
            }
        }

        stdout.write_all(b"\n").await?;
        stdout.flush().await?;

        Ok(None)
    }
}
