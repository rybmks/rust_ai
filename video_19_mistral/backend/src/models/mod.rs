use crate::Result;
use candle_core::Device;
use hf_hub::api::tokio::Api;
pub mod mistral7b;

pub trait Model {
    fn init(api: Api, device: Device) -> impl Future<Output = Result<impl Model>>;
    fn run<T: Into<String>>(&mut self, prompt: T, sample_len: usize) -> Result<()>;
}
