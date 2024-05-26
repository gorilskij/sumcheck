use ark_poly::Polynomial;
use ark_std::{One, UniformRand, Zero};

use super::{UVPoly, F};
use crate::channel::Channel;
use crate::oracle_once::OracleOnce;

pub struct Verifier {
    oracle: OracleOnce,
    degrees: Vec<usize>,
}

#[derive(Debug)]
pub enum Outcome {
    Accept,
    Reject(String),
}

impl Verifier {
    pub fn new(oracle: OracleOnce, degrees: Vec<usize>) -> Self {
        Self { oracle, degrees }
    }

    pub fn run_sumcheck(&mut self, ch: Channel) -> anyhow::Result<Outcome> {
        let H = ch.recv::<F>()?;

        let mut rng = ark_std::test_rng();
        let mut last_value = H;
        let mut fixed = vec![];
        for i in 0..self.degrees.len() {
            let q = ch.recv::<UVPoly>()?;

            if q.degree() > self.degrees[i] {
                return Ok(Outcome::Reject(format!("q has unexpectedly high degree")));
            }

            if q.evaluate(&F::zero()) + q.evaluate(&F::one()) != last_value {
                return Ok(Outcome::Reject(format!("q(0) + q(1) != last_value in round {i}")));
            }

            let r = F::rand(&mut rng);
            fixed.push(r);
            last_value = q.evaluate(&r);

            if i < self.degrees.len() - 1 {
                ch.send(r)?;
            } else {
                // last round
                let final_eval = self.oracle.evaluate(&fixed);
                if final_eval != last_value {
                    return Ok(Outcome::Reject(format!(
                        "final evaluation incorrect, expected {}, got {}",
                        last_value, final_eval,
                    )));
                }
            }
        }

        Ok(Outcome::Accept)
    }
}
