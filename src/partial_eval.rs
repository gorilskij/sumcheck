use std::collections::{HashMap, HashSet};

use ark_ff::{BigInt, Field};
use ark_poly::multivariate::{SparsePolynomial, SparseTerm, Term};
use ark_poly::DenseMVPolynomial;

pub trait PartialEval<F: Field> {
    fn partial_eval(&self, values: &HashMap<usize, F>) -> Self;
}

fn partial_eval_term<F: Field>(term: &SparseTerm, values: &HashMap<usize, F>) -> (F, SparseTerm) {
    let mut coef = F::one();
    let new_term = term
        .iter()
        .filter_map(|&(idx, exp)| match values.get(&idx) {
            Some(val) => {
                coef *= val.pow(BigInt::<1>::new([exp as u64]));
                None
            }
            None => Some((idx, exp)),
        })
        .collect();
    (coef, SparseTerm::new(new_term))
}

impl<F: Field> PartialEval<F> for SparsePolynomial<F, SparseTerm> {
    fn partial_eval(&self, values: &HashMap<usize, F>) -> Self {
        // cfg_into_iter!(&self.terms)
        //     .map(|(coeff, term)| *coeff * term.evaluate(point))
        //     .sum()

        let new_terms: Vec<_> = self
            .terms
            .iter()
            .map(|(coef, term)| {
                let (eval_coef, new_term) = partial_eval_term(term, values);
                (*coef * eval_coef, new_term)
            })
            .collect();

        // TODO: rename/renumber the variables and recalculate num_vars
        // let num_vars = new_terms.iter().flat_map(|(_, term)| term.iter().map(|(idx, _)| *idx)).collect::<HashSet<_>>().len();>
        // SparsePolynomial::from_coefficients_vec(num_vars, new_terms)

        SparsePolynomial::from_coefficients_vec(self.num_vars, new_terms)
    }
}

pub trait ToNum<F: Field> {
    fn to_num(&self) -> Option<F>;
}

impl<F: Field> ToNum<F> for SparsePolynomial<F, SparseTerm> {
    fn to_num(&self) -> Option<F> {
        match self.terms.len() {
            0 => Some(F::zero()),
            1 => match &self.terms[0].1.iter().next() {
                None => Some(self.terms[0].0),
                _ => None,
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::F;
    use ark_poly::multivariate::{SparsePolynomial, SparseTerm, Term};
    use ark_poly::DenseMVPolynomial;

    use super::PartialEval;

    #[test]
    fn test_partial_eval() {
        // 3 x0^2 x1^2 + 2 x1^3 x2 + 4 x2 + 7
        let poly = SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(3), SparseTerm::new(vec![(0, 2), (1, 2)])),
                (F::from(2), SparseTerm::new(vec![(1, 3), (2, 1)])),
                (F::from(4), SparseTerm::new(vec![(2, 1)])),
                (F::from(7), SparseTerm::new(vec![])),
            ],
        );

        let partial1 = poly.partial_eval(&map!(0, 3));
        let partial1_expected = SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(27), SparseTerm::new(vec![(1, 2)])),
                (F::from(2), SparseTerm::new(vec![(1, 3), (2, 1)])),
                (F::from(4), SparseTerm::new(vec![(2, 1)])),
                (F::from(7), SparseTerm::new(vec![])),
            ],
        );

        assert_eq!(partial1, partial1_expected);

        let partial2 = poly.partial_eval(&map!(1, 3));
        let partial2_expected = SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(27), SparseTerm::new(vec![(0, 2)])),
                (F::from(58), SparseTerm::new(vec![(2, 1)])),
                (F::from(7), SparseTerm::new(vec![])),
            ],
        );

        assert_eq!(partial2, partial2_expected);

        let partial3 = poly.partial_eval(&map!(2, 3));
        let partial3_expected = SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(3), SparseTerm::new(vec![(0, 2), (1, 2)])),
                (F::from(6), SparseTerm::new(vec![(1, 3)])),
                (F::from(19), SparseTerm::new(vec![])),
            ],
        );

        assert_eq!(partial3, partial3_expected);
    }
}
