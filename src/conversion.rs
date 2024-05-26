use std::collections::HashSet;

use ark_ff::Field;
use ark_poly::multivariate::{SparsePolynomial, SparseTerm};
use ark_poly::univariate::SparsePolynomial as UVSparsePolynomial;

fn real_num_vars<F: Field>(poly: &SparsePolynomial<F, SparseTerm>) -> usize {
    cfg_iter!(poly.terms)
        .flat_map(|(_, term)| term.iter().map(|(idx, _)| *idx))
        .collect::<HashSet<_>>()
        .len()
}

pub trait ToUnivariate<F: Field> {
    fn to_univariate(&self) -> Option<UVSparsePolynomial<F>>;
}

impl<F: Field> ToUnivariate<F> for SparsePolynomial<F, SparseTerm> {
    fn to_univariate(&self) -> Option<UVSparsePolynomial<F>> {
        if real_num_vars(self) > 1 {
            return None;
        }

        Some(UVSparsePolynomial::from_coefficients_vec(
            cfg_iter!(self.terms)
                .map(|(coef, term)| {
                    if term.len() == 0 {
                        (0, *coef)
                    } else {
                        (term.iter().next().unwrap().1, *coef)
                    }
                })
                .collect(),
        ))
    }
}

#[cfg(test)]
mod test {
    use ark_poly::multivariate::{SparsePolynomial, SparseTerm, Term};
    use ark_poly::univariate::SparsePolynomial as UVSparsePolynomial;
    use ark_poly::DenseMVPolynomial;

    use crate::conversion::ToUnivariate;
    use crate::F;

    #[test]
    fn test_to_univariate_fail() {
        let poly =
            SparsePolynomial::from_coefficients_vec(2, vec![(F::from(1), SparseTerm::new(vec![(0, 1), (1, 1)]))]);

        assert_eq!(poly.to_univariate(), None);

        let poly = SparsePolynomial::from_coefficients_vec(
            2,
            vec![
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
            ],
        );

        assert_eq!(poly.to_univariate(), None);
    }

    #[test]
    fn test_to_univariate_success() {
        let poly = SparsePolynomial::from_coefficients_vec(1, vec![(F::from(3), SparseTerm::new(vec![(0, 2)]))]);
        let expected = UVSparsePolynomial::from_coefficients_vec(vec![(2, F::from(3))]);

        assert_eq!(poly.to_univariate(), Some(expected));

        let poly = SparsePolynomial::from_coefficients_vec(
            4,
            vec![
                (F::from(3), SparseTerm::new(vec![(3, 4)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        );
        let expected = UVSparsePolynomial::from_coefficients_vec(vec![(4, F::from(3)), (0, F::from(5))]);

        assert_eq!(poly.to_univariate(), Some(expected));
    }
}
