use std::pin::Pin;

use crate::Result;
use candle_core::Device;
use futures::Stream;
use hf_hub::api::tokio::Api;

pub type ResponseStream<'a> = Pin<Box<dyn Stream<Item = Result<String>> + Send + Sync + 'a>>;

pub mod mistral7b;

pub trait Model {
    fn init(api: Api, device: Device) -> impl Future<Output = Result<impl Model + Send + Sync>>;
    fn run<T: Into<String>>(&mut self, prompt: T, sample_len: usize) -> Result<ResponseStream>;
}
