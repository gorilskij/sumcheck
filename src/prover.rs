use std::collections::HashMap;

use anyhow::Result;

use super::{Poly, F};
use crate::channel::Channel;
use crate::conversion::ToUnivariate;
use crate::partial_eval::{PartialEval, ToNum};

pub struct Prover {
    poly: Poly,
}

impl Prover {
    pub fn new(poly: Poly) -> Self {
        Self { poly }
    }

    /// `self.poly` summed over decreasing numbers of trailing variables.
    /// The first element is H, the next element is free only in x_0,
    /// the one after is free is x_0 and x_1, and so on until the last
    /// element which is free in all variables except x_n (i.e. the last
    /// element is equivalent to `[x_n := 0]self.poly + [x_n := 1]self.poly`)
    fn precompute_tails(&self) -> Vec<Poly> {
        assert!(self.poly.num_vars > 1);

        fn recur(poly: &Poly, idx: usize) -> Vec<Poly> {
            let p0 = poly.partial_eval(&map!(idx, 0));
            let p1 = poly.partial_eval(&map!(idx, 1));
            let mut tail = if idx == 0 {
                vec![]
            } else {
                recur(&p0, idx - 1)
                    .into_iter()
                    .zip(recur(&p1, idx - 1))
                    .map(|(v0, v1)| v0 + v1)
                    .collect()
            };
            tail.push(p0 + p1);
            tail
        }

        recur(&self.poly, self.poly.num_vars - 1)
    }

    pub fn run_sumcheck(&mut self, ch: Channel) -> Result<()> {
        let tails = self.precompute_tails();

        let H = tails[0].to_num().expect("H is not 0-variate");
        ch.send(H)?;

        let mut fixed = HashMap::new();
        for i in 0..self.poly.num_vars - 1 {
            let q = tails[i + 1]
                .partial_eval(&fixed)
                .to_univariate()
                .expect("q is not univariate");

            ch.send(q)?;
            let r = ch.recv::<F>()?;
            fixed.insert(i, r);
        }

        // last round
        let q = self
            .poly
            .partial_eval(&fixed)
            .to_univariate()
            .expect("q is not univariate in the last round");
        ch.send(q)?;

        Ok(())
    }
}
