macro_rules! map {
    ($key:expr, $val:expr) => {{
        let mut m = std::collections::HashMap::new();
        m.insert($key as usize, $val.into());
        m
    }};
}

mod conversion;
mod oracle_once;
mod partial_eval;
mod prover;
mod verifier;

use oracle_once::OracleOnce;
use prover::Prover;
use std::{sync::mpsc::channel, thread};
use verifier::Verifier;

use partial_eval::{PartialEval, ToNum};

use ark_bls12_381::Fq2 as F;
use ark_poly::{
    multivariate::{SparsePolynomial, SparseTerm, Term},
    univariate::SparsePolynomial as UVSparsePolynomial,
    DenseMVPolynomial, Polynomial,
};

use verifier::Outcome;

type Poly = SparsePolynomial<F, SparseTerm>;
type UVPoly = UVSparsePolynomial<F>;

enum Message {
    Value(F),
    MVPoly(Poly),
    UVPoly(UVPoly),
}

fn main() {
    // 3 x0^2 x1^2 + 2 x1^3 x2 + 4 x2 + 7
    let poly = SparsePolynomial::from_coefficients_vec(
        5,
        vec![
            (F::from(3), SparseTerm::new(vec![(0, 2), (1, 2)])),
            (F::from(2), SparseTerm::new(vec![(1, 3), (2, 1)])),
            (F::from(4), SparseTerm::new(vec![(2, 1)])),
            (F::from(7), SparseTerm::new(vec![])),
        ],
    );

    let poly_clone = poly.clone();

    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();
    let prover = thread::spawn(move || Prover::new(poly).run_sumcheck(tx1, rx2));
    let verifier = thread::spawn(move || {
        let num_vars = poly_clone.num_vars;
        let outcome = Verifier::new(OracleOnce::new(poly_clone), num_vars).run_sumcheck(tx2, rx1);
        if let Ok(Outcome::Reject(msg)) = &outcome {
            eprintln!("Verifier rejected with message:\n{msg:?}");
        }
        outcome
    });

    prover.join().unwrap().unwrap();
    let outcome = verifier.join().unwrap().unwrap();
    println!("{outcome:?}");
}
