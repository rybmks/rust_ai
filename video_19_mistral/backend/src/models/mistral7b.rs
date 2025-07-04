use std::io::Write;

use super::Model;
use crate::Result;
use candle_core::DType;
use candle_core::Device;
use candle_examples::token_output_stream::TokenOutputStream;
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::generation::Sampling;
use candle_transformers::models::mimi::candle_nn::VarBuilder;
use candle_transformers::models::mistral::{Config, Model as Mistral};
use futures::Stream;
use hf_hub::{Repo, RepoType, api::tokio::Api};
use tokenizers::Tokenizer;
use tokio::{fs::File, io::AsyncReadExt};

const MODEL_ID: &str = "mistralai/Mistral-7B-v0.1";
static DTYPE: DType = DType::F16;

pub struct Mistral7B {
    model: Mistral,
    device: Device,
    tokenizer: TokenOutputStream,
    logits_processor: LogitsProcessor,
}

impl Model for Mistral7B {
    fn init(api: Api, device: Device) -> impl std::future::Future<Output = Result<Mistral7B>> {
        let repo = api.repo(Repo::new(MODEL_ID.to_string(), RepoType::Model));

        async move {
            tracing::info!("Getting model files...");
            let mut filespath = vec![];

            let tokenizer_path = repo.get("tokenizer.json").await?;
            let config_path = repo.get("config.json").await?;
            let _tensors_index_path = repo.get("model.safetensors.index.json").await?;
            
            let _ = repo.get("tokenizer_config.json").await?;
            let tensors1 = repo.get("model-00001-of-00002.safetensors").await?;
            let tensors2 = repo.get("model-00002-of-00002.safetensors").await?;
            filespath.push(tensors1);
            filespath.push(tensors2);

            tracing::info!("Creating model config...");
            let mut config_file = File::open(config_path).await?;
            let mut config_buf = Vec::new();
            config_file.read_to_end(&mut config_buf).await?;
            let config: Config = serde_json::from_slice(&config_buf)?;

            tracing::info!("Creating model varmap...");
            let vb = unsafe { VarBuilder::from_mmaped_safetensors(&filespath, DTYPE, &device)? };
            tracing::info!("Creating model instance...");
            let model = Mistral::new(&config, vb)?;

            tracing::info!("Creating model tokenizer...");
            let tokenizer = Tokenizer::from_file(tokenizer_path)?;

            // TODO: configure model.
            let seed = 42; // random numbr generator seed (any number for reproducibility)
            let logits_processor = LogitsProcessor::from_sampling(
                seed,
                Sampling::TopK {
                    k: 10, // choose from the top-k most probable tokens
                    // Temperature controls the randomness of token sampling:
                    // - Less than 1.0 (e.g., 0.7): more confident, fewer options considered.
                    // - Greater than 1.0 (e.g., 1.2): more diverse, even low-probability tokens may be picked.
                    // - Close to 0.0: almost deterministic behavior, similar to argmax.
                    temperature: 0.7, // soft randomization (lower = more deterministic, higher = more creative)
                },
            );

            let model = Mistral7B {
                model,
                logits_processor,
                tokenizer: TokenOutputStream::new(tokenizer),
                device,
            };

            Ok(model)
        }
    }

    fn run<T: Into<String>>(&mut self, prompt: T, sample_len: usize) -> Result<()> {
        let prompt = format!(
            "<|system|>\nYou are an assistant. Answer concisely and finish your reply properly.\n<|user|>\n{}\n<|assistant|>\n",
            prompt.into()
        );

        let mut tokens = self
            .tokenizer
            .tokenizer()
            .encode(prompt, true)?
            .get_ids()
            .to_vec();

        for &t in tokens.iter() {
            if let Some(t) = self.tokenizer.next_token(t)? {
                print!("{t}")
            }
        }
        std::io::stdout().flush()?;
        let mut generated_tokens = 0usize;
        let eos_token = match self.tokenizer.get_token("</s>") {
            Some(token) => token,
            None => {
                tracing::error!("cannot find the </s> token");
                return Err(Box::from("cannot find the </s> token"));
            }
        };
        let start_gen = std::time::Instant::now();

        for index in 0..sample_len {
            let context_size = if index > 0 {
                1
            } else {
                usize::min(tokens.len(), 64)
            };
            let start_pos = tokens.len().saturating_sub(context_size);
            let ctxt = &tokens[start_pos..];
            let input = candle_core::Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, start_pos).map_err(|e| {
                Box::from(format!("Error while running forward: {e}"))
                    as Box<dyn std::error::Error + Send + Sync>
            })?;

            let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DTYPE)?;
            let next_token = self.logits_processor.sample(&logits)?;
            tokens.push(next_token);
            generated_tokens += 1;

            if next_token == eos_token {
                break;
            }

            if let Some(decoded) = self.tokenizer.decode_rest()?
                && (decoded.contains("<|end|>")
                    || decoded.contains("<|user|>")
                    || next_token == eos_token)
            {
                break;
            }

            if let Some(t) = self.tokenizer.next_token(next_token)? {
                print!("{t}");
                std::io::stdout().flush()?;
            }
        }
        let dt = start_gen.elapsed();
        if let Some(rest) = self.tokenizer.decode_rest()? {
            print!("{rest}");
        }
        std::io::stdout().flush()?;
        println!(
            "\n{generated_tokens} tokens generated ({:.2} token/s)",
            generated_tokens as f64 / dt.as_secs_f64(),
        );
        Ok(())
    }
}

impl Mistral7B {}
