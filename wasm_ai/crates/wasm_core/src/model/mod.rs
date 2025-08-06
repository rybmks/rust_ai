mod structs;

pub use structs::GgmlMetadata;
use wasmedge_wasi_nn::{GraphExecutionContext, TensorType};

pub async fn run_model(
    prompt: String,
    ctx: &mut GraphExecutionContext,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let input_index = 0;

    let input_data = prompt.as_bytes();
    let input_dims = &[input_data.len()];
    let input_type = TensorType::U8;

    ctx.set_input(input_index, input_type, input_dims, input_data)?;

    ctx.compute()?;
    let mut output = vec![0u8; 6048];
    ctx.get_output(0, &mut output)?;
    Ok(output)
}
