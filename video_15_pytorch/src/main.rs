use tch::Kind;
use tch::Tensor;
use tch::nn;
use tch::nn::LinearConfig;
use tch::nn::OptimizerConfig;
type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let device = tch::Device::Cpu;
    let vs = nn::VarStore::new(device);
    let linear = nn::linear(vs.root(), 1, 1, LinearConfig::default());
    let mut opt = nn::Sgd::default().build(&vs, 0.01)?;

    let x = Tensor::from_slice(&[1.0f32, 2.0, 3.0, 4.0]).view([4, 1]);
    let y_true = &x * 2.0 + 1.0;

    for epoch in 1..100 {
        let y_pred = x.apply(&linear);
        let loss = (&y_pred - &y_true).pow_tensor_scalar(2).sum(Kind::Float);
        opt.backward_step(&loss);

        if epoch % 10 == 0 {
            println!("Epoch: {epoch}, loss: {loss:?}",);
        }
    }

    let y_pred = x.apply(&linear);
    println!("Predictions: ");
    y_pred.print();
    println!("\nActual: ");
    y_true.print();
    Ok(())
}
