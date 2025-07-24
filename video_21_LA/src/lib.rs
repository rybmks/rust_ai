use std::fs;

#[derive(Debug)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Vec<f64>>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize) -> Matrix {
        Self {
            rows,
            cols,
            data: vec![vec![0.0; cols]; rows],
        }
    }

    pub fn from_file(path: &str) -> Matrix {
        let content = fs::read_to_string(path).unwrap_or_else(|e| panic!("{e}"));
        let matrix: Vec<Vec<f64>> = Vec::new();
        for rows in content.lines() {
            let mut row: Vec<f64> = vec![];
            let entries: Vec<&str> = rows.split_whitespace().collect();

            for ent in entries {
                row.push(ent.parse::<f64>().unwrap());
            }
        }
        let r = matrix.len();
        let c = matrix[0].len();
        Matrix {
            rows: r,
            cols: c,
            data: matrix,
        }
    }

    pub fn from_string(input: &str) -> Self {
        let mut data: Vec<Vec<f64>> = Vec::new();
        let rows: Vec<&str> = input.split(';').collect();

        for r in &rows {
            let entries: Vec<&str> = r.split_whitespace().collect();
            let mut tmp_row: Vec<f64> = Vec::new();
            for ent in entries {
                tmp_row.push(ent.parse::<f64>().unwrap());
            }
            data.push(tmp_row);
        }
        let n_r = data.len();
        let n_c = data[0].len();

        Self {
            rows: n_r,
            cols: n_c,
            data,
        }
    }

    pub fn copy(&self) -> Matrix {
        let mut data: Vec<Vec<f64>> = vec![];
        for row in &self.data {
            data.push(row.to_vec());
        }
        Matrix {
            rows: self.rows,
            cols: self.cols,
            data,
        }
    }

    pub fn print(&self) {
        self.data.iter().for_each(|v| println!("{v:?}"));
        println!()
    }

    pub fn identity(&mut self) {
        if self.rows != self.cols {
            panic!("Not a square matrix.");
        }

        for r in 0..self.rows {
            self.data[r][r] = 1.0;
        }
    }

    pub fn apply(&mut self, f: impl Fn(f64) -> f64) {
        self.data = self
            .data
            .iter()
            .map(|v| v.iter().map(|x| f(*x)).collect())
            .collect()
    }

    pub fn add(&self, b: Matrix) -> Matrix {
        if self.rows != b.rows || self.cols != b.cols {
            panic!("Matrices must be of the same size");
        }

        let mut sum = Matrix::new(self.rows, self.cols);
        for i in 0..self.rows {
            for j in 0..self.cols {
                sum.data[i][j] = self.data[i][j] + b.data[i][j];
            }
        }
        sum
    }

    pub fn substruct(&self, b: Matrix) -> Matrix {
        if self.rows != b.rows || self.cols != b.cols {
            panic!("Matrices must be of the same size");
        }

        let mut diff = Matrix::new(self.rows, self.cols);
        for i in 0..self.rows {
            for j in 0..self.cols {
                diff.data[i][j] = self.data[i][j] - b.data[i][j];
            }
        }
        diff
    }

    pub fn dot(&self, b: Matrix) -> Matrix {
        if self.rows != b.rows || self.cols != b.rows {
            panic!("Dimensions not matched")
        }
        let mut dp = Matrix::new(self.rows, b.cols);
        for i in 0..self.rows {
            for j in 0..b.cols {
                let mut sum = 0.0;
                for k in 0..self.cols {
                    sum += self.data[i][k] * b.data[k][j];
                }
                dp.data[i][j] = sum;
            }
        }
        dp
    }

    pub fn rref(&mut self) {
        if self.data[0][0] == 0.0 {
            swap_rows(self, 0);
        }
        let mut lead: usize = 0;
        let rows = self.rows;
        while lead < rows {
            for r in 0..rows {
                let div = self.data[lead][lead];
                let mult = self.data[r][lead] / div;

                if r == lead {
                    self.data[lead] = self.data[lead].iter().map(|entry| entry / div).collect();
                } else {
                    for c in 0..self.cols {
                        self.data[r][c] -= self.data[lead][c] * mult;
                    }
                }
                lead += 1;
            }
            correct(self)
        }
    }

    pub fn det(&self) -> f64 {
        if self.rows != self.cols {
            panic!("Determinant is only defined for square matrices");
        }
        if self.rows == 2 {
            return self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0];
        }

        let row: usize = 1;
        let mut det = 0.0;

        for j in 0..self.data[row].len() {
            det += self.cofactor(row, j) * self.data[row][j];
        }
        det
    }

    pub fn cofactor(&self, expanded_row: usize, j: usize) -> f64 {
        let cut: Vec<Vec<f64>> = vec![];
        for r in 0..self.rows {
            if r == expanded_row {
                continue;
            }

            let mut v: Vec<f64> = Vec::new();
            for c in 0..self.cols {
                if c == j {
                    continue;
                }
                v.push(self.data[r][c]);
            }
        }
        let n_r = cut.len();
        let n_c = cut[0].len();
        let minor = Matrix {
            rows: n_r,
            cols: n_c,
            data: cut,
        }
        .det();
        let base: i32 = -1;
        minor * f64::from(base.pow((expanded_row + j) as u32))
    }

    pub fn transpose(&self) -> Matrix {
        let mut transposed = Matrix::new(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                transposed.data[j][i] = self.data[i][j];
            }
        }
        transposed
    }

    pub fn inverse(&self) -> Self {
        let d = self.det();
        if d == 0.0 {
            panic!("Determinant is 0");
        }

        let mut inv = Self::new(self.rows, self.cols);

        for row in 0..self.rows {
            for col in 0..self.cols {
                inv.data[row][col] = self.cofactor(row, col);
            }
        }

        correct(&mut inv);
        inv = inv.transpose();
        inv.apply(|x| x / d);
        inv
    }
}

fn swap_rows(m: &mut Matrix, row: usize) {
    let mut n_r = 0;
    for r in 0..m.rows {
        if m.data[r][0] > 0.0 {
            n_r = r;
            break;
        }
    }
    let temp = m.data[row].clone();
    m.data[row] = m.data[n_r].clone();
    m.data[n_r] = temp;
}

fn correct(m: &mut Matrix) {
    for row in 0..m.rows {
        for col in 0..m.cols {
            let elem = m.data[row][col];
            if elem == -0.0 {
                m.data[row][col] = 0.0;
            }
            let floored = elem.floor();
            if elem - floored > 0.9999999 {
                m.data[row][col] = elem.round();
            }
            if elem > 0.0 && elem < 0.000001 {
                m.data[row][col] = 0.0;
            }
            if elem < 0.0 && elem > -0.00001 {
                m.data[row][col] = 0.0;
            }
        }
    }
}
