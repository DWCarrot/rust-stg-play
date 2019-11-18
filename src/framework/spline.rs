
use std::vec::Vec;
use num_traits::float::Float;


pub struct CubeSpline<T> {

    x_list: Vec<T>,

    y_list: Vec<T>,

    m_list: Vec<T>,

    dim: usize,

    c_ind: usize,

    c_mat: Vec<(T,T,T,T)>,

}

#[derive(Debug)]
pub enum CubeSplineError {
    GenInvalidInput,
    GenUnordered,
}



impl<T: Float> CubeSpline<T> {

    pub fn new() -> Self {
        CubeSpline {
            x_list: Vec::new(),
            y_list: Vec::new(),
            m_list: Vec::new(),
            dim: 0,
            c_ind: 0,
            c_mat: Vec::new(),
        }
    }

    ///
    /// x_list: vec n x 1   [x1, x2, ...]
    /// 
    /// y_list: vec n * d   [[y1,y2,...,yd], [y1,y2,...,yd], ...]
    pub fn compile(mut self, x_list: Vec<T>, y_list: Vec<T>) -> Result<Self, CubeSplineError> {
        let n = x_list.len() - 1;
        self.dim = y_list.len() / x_list.len();
        if x_list.len() < 3 || x_list.len() * self.dim < y_list.len() {
            return Err(CubeSplineError::GenInvalidInput);
        }
        
        let mut i: usize;
        let mut j: usize;

        let _2 = T::one() + T::one();
        let _6 = _2 + _2 + _2;

        self.m_list.resize(self.dim * (n + 1), T::zero());

        j = 0;
        while j < self.dim {
            let len_tmat = 3 * (n + 1) - 2;
            let mut tmat = Vec::new();
            tmat.resize(len_tmat, T::zero());
            let len_dvec = n + 1;
            let mut dvec = Vec::new();
            dvec.resize(len_dvec, T::zero());
            i = 1;
            let mut h_ = x_list[1] - x_list[0];
            let mut d_ = (y_list[self.dim + j] - y_list[j]) / h_;
            tmat[0] = T::one();
            tmat[len_tmat - 1] = T::one();
            while i < n {
                let h = x_list[i + 1] - x_list[i + 0];
                let d = (y_list[(i + 1) * self.dim + j] - y_list[i * self.dim + j]) / h;
                let i3 = i * 3;
                tmat[i3] = _2 * (h_ + h);
                tmat[i3 - 1] = h_;
                tmat[i3 + 1] = h;
                dvec[i] = _6 * (d - d_);
                i += 1;
                h_ = h;
                d_ = d;
            }
            self.solve_tridiag_mat(&mut tmat[3..len_tmat-3], &mut dvec[1..len_dvec-1]);
            i = 0;
            while i <= n {
                self.m_list[i * self.dim + j] = dvec[i];
                i += 1;
            }
            j += 1;
        }

        self.x_list = x_list;
        self.y_list = y_list;

        self.c_mat.resize(self.dim, (T::zero(), T::zero(), T::zero(), T::zero()));
        self.c_ind = 0;
        self.cal_cache();

        Ok(self)
    }

    pub fn get(&mut self, x: T, y: &mut [T]) -> usize {

        let mut x0 = self.x_list[self.c_ind];
        if x < x0 || x >= self.x_list[self.c_ind + 1] {
            x0 = self.binary_search(x);
            self.cal_cache();
        }

        let dx = x - x0;
        let dx2 = dx * dx;
        let dx3 = dx2 * dx;

        let mut j = 0;
        let dim = std::cmp::min(self.dim, y.len());
        while j < dim {
            let (a, b, c, d) = self.c_mat[j];
            y[j] = a + b * dx + c * dx2 + d * dx3;
            j += 1;
        }
        dim
    }

    pub fn get_derivative(&mut self, x: T, y: &mut [T]) -> usize {
        let mut x0 = self.x_list[self.c_ind];
        if x < x0 || x >= self.x_list[self.c_ind + 1] {
            x0 = self.binary_search(x);
            self.cal_cache();
        }

        let dx = x - x0;
        let dx2 = dx * dx;
        //let dx3 = dx2 * dx;

        let _2 = T::one() + T::one();
        let _3 = _2 + T::one();

        let mut j = 0;
        let dim = std::cmp::min(self.dim, y.len());
        while j < dim {
            let (_a, b, c, d) = self.c_mat[j];
            y[j] = b  + _2 * c * dx + _3 * d * dx2;
            j += 1;
        }
        dim
    }

    pub fn get_derivative2(&mut self, x: T, y: &mut [T]) -> usize {
        let mut x0 = self.x_list[self.c_ind];
        if x < x0 || x >= self.x_list[self.c_ind + 1] {
            x0 = self.binary_search(x);
            self.cal_cache();
        }

        let dx = x - x0;
        //let dx2 = dx * dx;
        //let dx3 = dx2 * dx;

        let _2 = T::one() + T::one();
        let _6 = (_2 + T::one()) * _2;

        let mut j = 0;
        let dim = std::cmp::min(self.dim, y.len());
        while j < dim {
            let (_a, _b, c, d) = self.c_mat[j];
            y[j] = _2 * c + _6 * d * dx;
            j += 1;
        }
        dim
    }

    fn binary_search(&mut self, x: T) -> T {
        let mut low = 1;
        if x < self.x_list[low] {
            self.c_ind = 0;
            return self.x_list[self.c_ind];
        }
        let mut high = self.x_list.len() - 2;
        if x > self.x_list[high] {
            self.c_ind = high;
            return self.x_list[self.c_ind];
        }
        while high - low > 1 {
            let mid = (low + high) / 2;
            let xm = self.x_list[mid];
            if xm > x {
                high = mid;
                continue;
            }
            if xm < x {
                low = mid;
                continue;
            }
            low = mid;
            break;
        }
        self.c_ind = low;
        return self.x_list[self.c_ind];
    }

    fn solve_tridiag_mat(&self, tmat: &mut [T], dvec: &mut [T]) {
        let mut i = 3;
        let mut j = 1;
        while i < tmat.len() {
            let r = tmat[i - 1] / tmat[i - 3];
            //tmat[i - 1] = T::zero();
            tmat[i] = tmat[i] - r * tmat[i - 2];
            //tmat[i + 1] = tmat[i + 1];
            dvec[j] = dvec[j] - r * dvec[j - 1];
            i += 3;
            j += 1;
        }
        i -= 3;
        j -= 1;
        dvec[j] = dvec[j] / tmat[i];
        loop {
            i -= 3;
            j -= 1;
            dvec[j] = (dvec[j] - tmat[i + 1] * dvec[j + 1]) / tmat[i];

            if i == 0 { break; }
        } 

    }

    fn cal_cache(&mut self) {

        let _2 = T::one() + T::one();
        let _6 = _2 + _2 + _2;
        let mut j = 0;
        while j < self.dim {
            let y0 = self.y_list[self.c_ind * self.dim + j];
            let y1 = self.y_list[(self.c_ind + 1) * self.dim + j];
            let m0 = self.m_list[self.c_ind * self.dim + j];
            let m1 = self.m_list[(self.c_ind + 1) * self.dim + j];
            let h0 = self.x_list[self.c_ind + 1] - self.x_list[self.c_ind];
            self.c_mat[j] = (
                y0,
                (y1 - y0) / h0 - h0 / _2 * m0 - h0 / _6 * (m1 - m0),
                m0 / _2,
                (m1 - m0) / (_6 * h0)
            );
            j += 1;
        }
        
    }


}






// use float::Float;

// fn binary_search<'a, T: Float>(knot: &T, ts: &'a [T]) -> &'a[T] {
//     let mut i = 0;
//     let mut j = ts.len() - 1;
//     while j - i > 1 {
//         let h = (i + j) / 2;
//         let vh = &ts[h];
//         if vh <= knot {
//             i = h;
//         } else {
//             j = h;
//         }
//     }
//     &ts[i-1..j+1]
// }

// fn catmull_rom_find<T: Float>(target: T, knots: &[T]) -> T {
//     let _1_0 = T::one();
//     let _2_0 = _1_0 + _1_0;
//     let _3_0 = _2_0 + _1_0;
//     let esp = (_1_0 / _2_0).powf(_3_0);

//     let tau = _1_0 / _2_0;

//     let mut t = _1_0 / _2_0;
//     loop {
//         let t_1 = t;
//         let t_2 = t_1 * t_1;
//         let t_3 = t_2 * t_1;
//         let d = knots[1];
//         let c = - tau * knots[0] + tau * knots[2];
//         let b = _2_0 * tau * knots[0] + (tau - _3_0) * knots[1] + (_3_0 - _2_0 * tau) * knots[2] - tau * knots[3];
//         let a = - tau * knots[0] + (_2_0 - tau) * knots[1] + (tau - _2_0) * knots[2] + tau * knots[3];
//         let step = t_1 - (a * t_3 + b * t_2 + c * t_1 + d - target) / (_3_0 * a * t_2 + _2_0 * b * t_1 + c);
//         t += step;
//         if step < esp {
//             break;
//         }
//     }
//     t
// }

// fn catmull_rom_weight<T: Float>(t: T) -> (T, T, T, T) {
    
//     let _1_0 = T::one();
//     let _2_0 = _1_0 + _1_0;
//     let _3_0 = _2_0 + _1_0;

//     let tau = _1_0 / _2_0;

//     let t_1 = t;
//     let t_2 = t_1 * t_1;
//     let t_3 = t_2 * t_1;

//     let w1 = - tau * t_3 + _2_0 * tau * t_2 - tau * t_1;
//     let w2 = (_2_0 - tau) * t_3 + (tau - _3_0) * t_2 + _1_0;
//     let w3 = (tau - _2_0) * t_3 + (_3_0 - _2_0 * tau) * t_2 + tau * t_1;
//     let w4 = tau * t_3 - tau * t_2;

//     (w1, w2, w3, w4)
// }

