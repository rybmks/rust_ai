//! Source: https://github.com/LlamaEdge/LlamaEdge/blob/main/crates/llama-core/src/metadata/ggml.rs#L162

use serde::{Deserialize, Serialize};

pub const DEFAULT_MODEL_NAME: &str = "tinyllama-1.1b-chat-v1.0.Q5_K_M.gguf";
pub const DEFAULT_MODEL_ALIAS: &str = "tinyllama";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GgmlMetadata {
    // this field not defined for the beckend plugin
    #[serde(skip_serializing)]
    pub model_name: String,
    // this field not defined for the beckend plugin
    #[serde(skip_serializing)]
    pub model_alias: String,
    // * Plugin parameters (used by this plugin):
    #[serde(rename = "enable-log")]
    pub log_enable: bool,
    #[serde(rename = "enable-debug-log")]
    pub debug_log: bool,
    // #[serde(rename = "stream-stdout")]
    // pub stream_stdout: bool,
    #[serde(rename = "embedding")]
    pub embeddings: bool,
    /// Number of tokens to predict, -1 = infinity, -2 = until context filled. Defaults to -1.
    #[serde(rename = "n-predict")]
    pub n_predict: i32,
    /// Halt generation at PROMPT, return control in interactive mode.
    #[serde(skip_serializing_if = "Option::is_none", rename = "reverse-prompt")]
    pub reverse_prompt: Option<String>,
    /// path to the multimodal projector file for llava
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mmproj: Option<String>,
    /// Path to the image file for llava
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    // * Model parameters (need to reload the model if updated):
    #[serde(rename = "n-gpu-layers")]
    pub n_gpu_layers: u64,
    /// The main GPU to use. Defaults to None.
    #[serde(rename = "main-gpu")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_gpu: Option<u64>,
    /// How split tensors should be distributed accross GPUs. If None the model is not split; otherwise, a comma-separated list of non-negative values, e.g., "3,2" presents 60% of the data to GPU 0 and 40% to GPU 1. Defaults to None.
    #[serde(rename = "tensor-split")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tensor_split: Option<String>,
    /// Whether to use memory-mapped files for the model. Defaults to `true`.
    #[serde(skip_serializing_if = "Option::is_none", rename = "use-mmap")]
    pub use_mmap: Option<bool>,
    /// How to split the model across multiple GPUs. Possible values:
    /// - `none`: use one GPU only
    /// - `layer`: split layers and KV across GPUs (default)
    /// - `row`: split rows across GPUs
    #[serde(rename = "split-mode")]
    pub split_mode: String,

    // * Context parameters (used by the llama context):
    /// Size of the prompt context. 0 means loaded from model. Defaults to 4096.
    #[serde(rename = "ctx-size")]
    pub ctx_size: u64,
    /// Logical maximum batch size. Defaults to 2048.
    #[serde(rename = "batch-size")]
    pub batch_size: u64,
    /// Physical maximum batch size. Defaults to 512.
    #[serde(rename = "ubatch-size")]
    pub ubatch_size: u64,
    /// Number of threads to use during generation. Defaults to 2.
    #[serde(rename = "threads")]
    pub threads: u64,

    // * Sampling parameters (used by the llama sampling context).
    /// Adjust the randomness of the generated text. Between 0.0 and 2.0. Defaults to 0.8.
    #[serde(rename = "temp")]
    pub temperature: f64,
    /// Top-p sampling. Between 0.0 and 1.0. Defaults to 0.9.
    #[serde(rename = "top-p")]
    pub top_p: f64,
    /// Penalize repeat sequence of tokens. Defaults to 1.0.
    #[serde(rename = "repeat-penalty")]
    pub repeat_penalty: f64,
    /// Repeat alpha presence penalty. Defaults to 0.0.
    #[serde(rename = "presence-penalty")]
    pub presence_penalty: f64,
    /// Repeat alpha frequency penalty. Defaults to 0.0.
    #[serde(rename = "frequency-penalty")]
    pub frequency_penalty: f64,

    // * grammar parameters
    /// BNF-like grammar to constrain generations (see samples in grammars/ dir). Defaults to empty string.
    pub grammar: String,
    /// JSON schema to constrain generations (<https://json-schema.org/>), e.g. `{}` for any JSON object. For schemas w/ external $refs, use --grammar + example/json_schema_to_grammar.py instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<String>,

    /// Whether to include usage in the stream response. Defaults to false.
    pub include_usage: bool,
}
impl Default for GgmlMetadata {
    fn default() -> Self {
        Self {
            model_name: String::from(DEFAULT_MODEL_NAME),
            model_alias: String::from(DEFAULT_MODEL_ALIAS),
            debug_log: false,
            log_enable: false,
            embeddings: false,
            n_predict: -1,
            reverse_prompt: Some("User:".to_string()),
            mmproj: None,
            image: None,
            n_gpu_layers: 100,
            main_gpu: None,
            tensor_split: None,
            use_mmap: Some(true),
            split_mode: "layer".to_string(),
            ctx_size: 4096,
            batch_size: 1024,
            ubatch_size: 16,
            threads: 2,
            temperature: 0.8,
            top_p: 0.9,
            repeat_penalty: 1.2,
            presence_penalty: 0.0,
            frequency_penalty: 0.0,
            grammar: String::new(),
            json_schema: None,
            include_usage: false,
        }
    }
}
