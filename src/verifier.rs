use std::sync::mpsc::{Receiver, Sender};

use ark_ff::Field;
use ark_poly::Polynomial;

use crate::{oracle_once::OracleOnce, Message};

use super::{Poly, UVPoly, F};
use ark_std::{One, UniformRand, Zero};

pub struct Verifier {
    oracle: OracleOnce,
    num_vars: usize,
}

#[derive(Debug)]
pub enum Outcome {
    Accept,
    Reject(String),
}

impl Verifier {
    pub fn new(oracle: OracleOnce, num_vars: usize) -> Self {
        Self { oracle, num_vars }
    }

    pub fn run_sumcheck(
        &mut self,
        tx: Sender<Message>,
        rx: Receiver<Message>,
    ) -> anyhow::Result<Outcome> {
        // TODO: error handling
        let Message::Value(H) = rx.recv()? else {
            panic!()
        };

        let mut rng = ark_std::test_rng();
        let mut last_value = H;
        let mut fixed = vec![];
        for i in 0..self.num_vars - 1 {
            let Message::UVPoly(q) = rx.recv()? else {
                panic!()
            };

            // TODO: verify polynomial
            if q.evaluate(&F::zero()) + q.evaluate(&F::one()) != last_value {
                return Ok(Outcome::Reject(format!(
                    "q(0) + q(1) != last_value in round {i}"
                )));
            }

            let r = F::rand(&mut rng);
            fixed.push(r);
            last_value = q.evaluate(&r);
            tx.send(Message::Value(r))?;
        }

        // last round
        let Message::UVPoly(q) = rx.recv()? else {
            panic!()
        };

        // TODO: verify polynomial
        if q.evaluate(&F::zero()) + q.evaluate(&F::one()) != last_value {
            return Ok(Outcome::Reject(
                "q(0) + q(1) != last_value in the last round".to_string(),
            ));
        }

        let r = F::rand(&mut rng);
        fixed.push(r);
        last_value = q.evaluate(&r);

        let final_eval = self.oracle.evaluate(&fixed);
        if final_eval != last_value {
            return Ok(Outcome::Reject(format!(
                "final evaluation incorrect, expected {}, got {}",
                last_value, final_eval,
            )));
        }

        Ok(Outcome::Accept)
    }
}
