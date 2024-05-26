macro_rules! map {
    ($key:expr, $val:expr) => {{
        let mut m = std::collections::HashMap::new();
        m.insert($key as usize, $val.into());
        m
    }};
}

#[macro_use]
extern crate ark_std as _;

mod channel;
mod conversion;
mod oracle_once;
mod partial_eval;
mod prover;
mod verifier;

use std::cmp::max;
use std::collections::HashMap;
use std::thread;

use ark_bls12_381::Fq2 as F;
use ark_poly::multivariate::{SparsePolynomial, SparseTerm, Term};
use ark_poly::univariate::SparsePolynomial as UVSparsePolynomial;
use ark_poly::DenseMVPolynomial;
use oracle_once::OracleOnce;
use prover::Prover;
use verifier::{Outcome, Verifier};

use crate::channel::Channel;

type Poly = SparsePolynomial<F, SparseTerm>;
type UVPoly = UVSparsePolynomial<F>;

fn main() {
    // 3 x0 x1 x2 x3 x4
    // 4 x0 x2 x4
    // 5 x1 x2 x3
    // 7 x0 x1
    // 2 x0 x4
    // 8 x1 x2
    // 17 x2 x3
    // 12 x1
    // 3 x4
    // 9
    let poly = SparsePolynomial::from_coefficients_vec(
        5,
        vec![
            (
                F::from(3),
                SparseTerm::new(vec![(0, 1), (1, 1), (2, 1), (3, 1), (4, 1)]),
            ),
            (F::from(4), SparseTerm::new(vec![(0, 1), (2, 1), (4, 1)])),
            (F::from(5), SparseTerm::new(vec![(1, 1), (2, 1), (3, 1)])),
            (F::from(7), SparseTerm::new(vec![(0, 1), (1, 1)])),
            (F::from(2), SparseTerm::new(vec![(0, 1), (4, 1)])),
            (F::from(8), SparseTerm::new(vec![(1, 1), (2, 1)])),
            (F::from(17), SparseTerm::new(vec![(2, 1), (3, 1)])),
            (F::from(12), SparseTerm::new(vec![(1, 1)])),
            (F::from(3), SparseTerm::new(vec![(4, 1)])),
            (F::from(9), SparseTerm::new(vec![])),
        ],
    );
    let poly_clone = poly.clone();

    let (ch1, ch2) = Channel::new_pair();
    let prover = thread::spawn(move || Prover::new(poly).run_sumcheck(ch1));
    let verifier = thread::spawn(move || {
        let num_vars = poly_clone.num_vars;
        let degrees = {
            let mut degrees = HashMap::new();
            cfg_iter!(poly_clone.terms).for_each(|(_, term)| {
                term.iter().for_each(|&(i, exp)| {
                    degrees
                        .entry(i)
                        .and_modify(|current| *current = max(*current, exp))
                        .or_insert(exp);
                })
            });
            (0..num_vars).map(|i| degrees.get(&i).copied().unwrap_or(0)).collect()
        };
        let outcome = Verifier::new(OracleOnce::new(poly_clone), degrees).run_sumcheck(ch2);
        if let Ok(Outcome::Reject(msg)) = &outcome {
            eprintln!("Verifier rejected with message:\n{msg:?}");
        }
        outcome
    });

    prover.join().unwrap().unwrap();
    let outcome = verifier.join().unwrap().unwrap();
    println!("{outcome:?}");
}
