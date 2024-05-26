use ark_poly::Polynomial;

use super::{Poly, F};

pub struct OracleOnce(Option<Poly>);

impl OracleOnce {
    pub fn new(poly: Poly) -> Self {
        Self(Some(poly))
    }

    pub fn evaluate(&mut self, point: &Vec<F>) -> F {
        self.0.take().expect("attempted repeated evaluation").evaluate(point)
    }
}
