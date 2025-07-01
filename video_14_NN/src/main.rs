use std::f64::consts::E;
use video_14_NN::*;

pub const SIGMOID: Activation = Activation {
    function: |x| 1.0 / (1.0 + E.powf(-x)),
    derivative: |x| x * (1.0 - x),
};

fn main() {
    let inputs = vec![
        vec![0.0, 0.0],
        vec![0.0, 1.0],
        vec![1.0, 0.0],
        vec![1.0, 1.0],
    ];
    let targets = vec![vec![0.0], vec![1.0], vec![0.0], vec![1.0]];

    let mut network = Network::new(vec![2, 3, 1], SIGMOID, 0.5);

    network.train(inputs, targets, 100000);

    println!("{:?}", network.feed_forward(Matrix::from(vec![0.0, 0.0])));
    println!("{:?}", network.feed_forward(Matrix::from(vec![0.0, 1.0])));
    println!("{:?}", network.feed_forward(Matrix::from(vec![1.0, 0.0])));
    println!("{:?}", network.feed_forward(Matrix::from(vec![1.0, 1.0])));
}
