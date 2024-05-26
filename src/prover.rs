use crate::{
    conversion::ToUnivariate,
    partial_eval::{PartialEval, ToNum},
    Message,
};
use anyhow::Result;
use ark_poly::Polynomial;
use ark_std::Zero;
use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use super::{Poly, UVPoly, F};

pub struct Prover {
    poly: Poly,
}

impl Prover {
    pub fn new(poly: Poly) -> Self {
        Self { poly }
    }

    fn compute_H(&self) -> F {
        fn recur(poly: &Poly, num_vars: usize) -> F {
            if num_vars == 0 {
                poly.to_num().expect("expected 0-variate polynomial")
            } else {
                let idx = num_vars - 1;
                let poly_0 = poly.partial_eval(&map!(idx, 0));
                let poly_1 = poly.partial_eval(&map!(idx, 1));
                recur(&poly_0, idx) + recur(&poly_1, idx)
            }
        }

        recur(&self.poly, self.poly.num_vars)
    }

    // // TODO: integrate into run_sumcheck to avoid recomputing every time
    // // `fixed` represents the fixed values at the start
    // fn compute_univariate(&self, fixed: &[F]) -> UVPoly {
    //     fn recur(
    //         poly: &Poly,
    //         num_vars: usize,
    //         idx: usize,
    //         fixed: &[F],
    //         free_var_inserted: bool,
    //     ) -> Poly {
    //         match fixed {
    //             &[r, ref fixed @ ..] => recur(
    //                 &poly.partial_eval(&map!(idx, r)),
    //                 num_vars,
    //                 idx + 1,
    //                 fixed,
    //                 false,
    //             ),
    //             _ if !free_var_inserted => recur(poly, num_vars, idx + 1, fixed, true),
    //             _ => {
    //                 let poly_0 = poly.partial_eval(&map!(idx, 0));
    //                 let poly_1 = poly.partial_eval(&map!(idx, 1));
    //                 if idx < num_vars {
    //                     recur(&poly_0, num_vars, idx + 1, &[], true)
    //                         + recur(&poly_1, num_vars, idx + 1, &[], true)
    //                 } else {
    //                     poly_0 + poly_1
    //                 }
    //             }
    //         }
    //     }

    //     let tmp = recur(&self.poly, self.poly.num_vars, 0, fixed, false);
    //     tmp.to_univariate().expect("expected univariate polynomial")
    // }

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

    pub fn run_sumcheck(&mut self, tx: Sender<Message>, rx: Receiver<Message>) -> Result<()> {
        let precomputed = self.precompute_tails();

        let H = precomputed[0].to_num().expect("H is not 0-variate");
        tx.send(Message::Value(H))?;

        let mut fixed = HashMap::new();
        for i in 0..self.poly.num_vars - 1 {
            // let univariate = self.compute_univariate(&fixed);
            let q = precomputed[i + 1]
                .partial_eval(&fixed)
                .to_univariate()
                .expect("q is not univariate");

            // println!("{:?}\n===>\n{:?}", self.poly, univariate);

            tx.send(Message::UVPoly(q))?;
            let r = rx.recv()?;
            // TODO: error handling
            let Message::Value(r) = r else { panic!() };
            fixed.insert(i, r);
        }

        // last round
        // let point: Vec<_> = (0..).map_while(|i| fixed.get(&i)).copied().collect();
        // assert_eq!(point.len(), self.poly.num_vars);
        let q = self
            .poly
            .partial_eval(&fixed)
            .to_univariate()
            .expect("q is not univariate in the last round");
        tx.send(Message::UVPoly(q))?;

        Ok(())
    }
}
