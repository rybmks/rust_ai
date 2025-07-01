use burn::backend::Autodiff;
use burn::module::Module;
use burn::nn::{Linear, LinearConfig};
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use burn::prelude::Backend;
use burn::tensor::Tensor;

type Back = Autodiff<NdArray<f32>>;
// type Back = Wgpu<f32>;

use burn::nn::Relu;
use burn_ndarray::NdArray;
use rand::Rng;

#[derive(Module, Debug)]
#[module(params)]
pub struct Model<B: Backend> {
    linear1: Linear<B>,
    activation: Relu,
    linear2: Linear<B>,
}

impl<B: Backend> Model<B> {
    pub fn init(device: &B::Device) -> Model<B> {
        Self {
            linear1: LinearConfig::new(1, 4).init(device),
            activation: Relu::new(),
            linear2: LinearConfig::new(4, 1).init(device),
        }
    }

    pub fn forward(&self, x: burn::tensor::Tensor<B, 2>) -> burn::tensor::Tensor<B, 2> {
        let x = self.linear1.forward(x);
        let x = self.activation.forward(x);
        self.linear2.forward(x)
    }
}

fn main() {
    let mut optimizer = AdamConfig::new().init::<Back, Model<Back>>();

    let device = <Back as Backend>::Device::default();
    let mut model: Model<Back> = Model::init(&device);

    // Training data
    let mut rng = rand::rng();
    let inputs: Vec<f32> = (0..100).map(|_| rng.random_range(0.0..1.0)).collect();
    let targets: Vec<f32> = inputs.iter().map(|x| 2.0 * x + 1.0).collect();

    let inputs: [[f32; 1]; 100] = inputs
        .into_iter()
        .map(|x| [x])
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let targets: [[f32; 1]; 100] = targets
        .into_iter()
        .map(|y| [y])
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let x = Tensor::<Back, 2>::from_floats(inputs, &device);
    let y_true = Tensor::<Back, 2>::from_floats(targets, &device);

    for epoch in 0..1000 {
        let y_pred = model.forward(x.clone());
        let loss = (y_pred.clone() - y_true.clone()).powf_scalar(2.0).mean();
        let grads = loss.backward();
        let learning_rate = 1e-3;
        let grads = GradientsParams::from_grads(grads, &model);
        model = optimizer.step(learning_rate, model, grads);
        if epoch % 100 == 0 {
            println!("Epoch {epoch}: loss = {:?}", loss.to_data());
        }
    }

    let x_test = Tensor::<Back, 2>::from_floats([[0.5]], &device);
    let y_test = model.forward(x_test);

    let y_test_data = y_test.to_data();
    let y_test_vec: Vec<f32> = y_test_data.to_vec().unwrap();

    println!("Prediction for x = 0.5: {}", y_test_vec[0]);
}

#[allow(unused)]
pub fn tensor_creation_example() {
    let device = Default::default();
    // Creation of two tensors, the first with explicit values and the second one with ones, with the same shape as the first
    let tensor_1 = Tensor::<Back, 2>::from_data([[2., 3.], [4., 5.]], &device);
    let tensor_2 = Tensor::<Back, 2>::ones_like(&tensor_1);

    // Print the element-wise addition (done with the WGPU backend) of the two tensors.
    println!("{}", tensor_1 + tensor_2);
}
