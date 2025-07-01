use rand::Rng;

#[derive(Debug, Clone)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

impl Matrix {
    pub fn get_random(rows: usize, cols: usize) -> Self {
        let mut buffer = Vec::with_capacity(rows * cols);

        let mut rng = rand::rng();
        for _ in 0..rows * cols {
            buffer.push(rng.random_range(0.0..1.0));
        }
        Matrix {
            rows,
            cols,
            data: buffer,
        }
    }

    pub fn add(&self, other: &Matrix) -> Matrix {
        assert_eq!(self.rows, other.rows, "Matrix row counts must match");
        assert_eq!(self.cols, other.cols, "Matrix column counts must match");

        let data = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a + b)
            .collect();

        Matrix {
            rows: self.rows,
            cols: self.cols,
            data,
        }
    }

    pub fn substract(&self, other: &Matrix) -> Matrix {
        assert_eq!(self.rows, other.rows, "Matrix row counts must match");
        assert_eq!(self.cols, other.cols, "Matrix column counts must match");

        let data = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a - b)
            .collect();

        Matrix {
            rows: self.rows,
            cols: self.cols,
            data,
        }
    }

    pub fn elementwise_multiply(&self, other: &Matrix) -> Matrix {
        if self.rows != other.rows || self.cols != other.cols {
            panic!("Attempted to multiply by matrix of incorrect dimensions");
        }

        let mut result_data = vec![0.0; self.cols * self.rows];
        for (i, (a, b)) in self.data.iter().zip(other.data.iter()).enumerate() {
            result_data[i] = a * b;
        }

        Matrix {
            rows: self.rows,
            cols: self.cols,
            data: result_data,
        }
    }

    pub fn dot_multiply(&self, other: &Matrix) -> Matrix {
        assert_eq!(self.cols, other.rows);
        let mut result_data = vec![0.0; self.rows * other.cols];

        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut sum = 0.0;
                for k in 0..self.cols {
                    sum += self.data[i * self.cols + k] * other.data[k * other.cols + j];
                }
                result_data[i * other.cols + j] = sum;
            }
        }
        Matrix {
            rows: self.rows,
            cols: other.cols,
            data: result_data,
        }
    }

    pub fn transpose(&self) -> Matrix {
        let mut buffer = vec![0.0; self.cols * self.rows];

        for i in 0..self.rows {
            for j in 0..self.cols {
                buffer[j * self.rows + i] = self.data[i * self.cols + j];
            }
        }

        Matrix {
            rows: self.cols,
            cols: self.rows,
            data: buffer,
        }
    }

    pub fn map(&mut self, func: fn(&f64) -> f64) -> Matrix {
        let mut result = Matrix {
            rows: self.rows,
            cols: self.cols,
            data: Vec::with_capacity(self.data.len()),
        };

        result.data.extend(self.data.iter().map(|&val| func(&val)));

        result
    }
}

impl From<Vec<f64>> for Matrix {
    fn from(vec: Vec<f64>) -> Self {
        let rows = vec.len();
        let cols = 1;
        Matrix {
            rows,
            cols,
            data: vec,
        }
    }
}

pub struct Network {
    layers: Vec<usize>,
    weights: Vec<Matrix>,
    biases: Vec<Matrix>,
    data: Vec<Matrix>,
    activation: Activation,
    learning_rate: f64,
}

pub struct Activation {
    pub function: fn(&f64) -> f64,
    pub derivative: fn(&f64) -> f64,
}

impl Network {
    pub fn new(layers: Vec<usize>, activation: Activation, learning_rate: f64) -> Self {
        let mut weights = vec![];
        let mut biases = vec![];

        for i in 0..layers.len() - 1 {
            weights.push(Matrix::get_random(layers[i + 1], layers[i]));
            biases.push(Matrix::get_random(layers[i + 1], 1));
        }

        Network {
            layers,
            weights,
            biases,
            data: vec![],
            activation,
            learning_rate,
        }
    }

    pub fn feed_forward(&mut self, inputs: Matrix) -> Matrix {
        assert!(
            self.layers[0] == inputs.data.len(),
            "Invalid number of inputs"
        );

        let mut current = inputs;
        self.data = vec![current.clone()];

        for i in 0..self.layers.len() - 1 {
            current = self.weights[i]
                .dot_multiply(&current)
                .add(&self.biases[i])
                .map(self.activation.function);

            self.data.push(current.clone());
        }

        current
    }

    pub fn back_propogate(&mut self, inputs: Matrix, targets: Matrix) {
        let mut errors: Matrix = targets.substract(&inputs);
        let mut gradients = inputs.clone().map(self.activation.derivative);

        for i in (0..self.layers.len() - 1).rev() {
            gradients = gradients.elementwise_multiply(&errors).map(|x| x * 0.5);
            self.weights[i] =
                self.weights[i].add(&gradients.dot_multiply(&self.data[i].transpose()));
            self.biases[i] = self.biases[i].add(&gradients);
            errors = self.weights[i].transpose().dot_multiply(&errors);
            gradients = self.data[i].map(self.activation.derivative);
        }
    }

    pub fn train(&mut self, inputs: Vec<Vec<f64>>, targets: Vec<Vec<f64>>, epochs: u32) {
        for i in 1..=epochs {
            if epochs < 100 || i % (epochs / 100) == 0 {
                println!("Epoch {i} of {epochs}");
            }
            for j in 0..inputs.len() {
                let outputs = self.feed_forward(Matrix::from(inputs[j].clone()));
                self.back_propogate(outputs, Matrix::from(targets[j].clone()));
            }
        }
    }
}
