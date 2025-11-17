//! Linear Algebra Utilities
//!
//! Hilfsfunktionen für Matrix-Operationen

use nalgebra::{SMatrix, SVector};
use num_complex::Complex64;

/// Matrix-Exponentiation via Eigenwertzerlegung
///
/// Berechnet exp(A) für hermitesche Matrix A (nur für kleine fixe Dimensionen)
pub fn matrix_exp_hermitian_3x3(matrix: &SMatrix<f64, 3, 3>) -> SMatrix<Complex64, 3, 3> {
    use nalgebra::SymmetricEigen;

    let eigen = SymmetricEigen::new(*matrix);

    let mut result = SMatrix::<Complex64, 3, 3>::zeros();

    for i in 0..3 {
        let exp_eigenvalue = eigen.eigenvalues[i].exp();

        for j in 0..3 {
            for k in 0..3 {
                result[(j, k)] += Complex64::from(
                    exp_eigenvalue * eigen.eigenvectors[(j, i)] * eigen.eigenvectors[(k, i)],
                );
            }
        }
    }

    result
}

/// Berechnet Frobenius-Norm einer Matrix
pub fn frobenius_norm<const N: usize>(matrix: &SMatrix<Complex64, N, N>) -> f64 {
    matrix.iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt()
}

/// Prüft ob Matrix unitär ist: U†U = I
pub fn is_unitary<const N: usize>(matrix: &SMatrix<Complex64, N, N>, tol: f64) -> bool {
    let product = matrix.adjoint() * matrix;
    let identity = SMatrix::<Complex64, N, N>::identity();

    let diff = product - identity;
    frobenius_norm(&diff) < tol
}

/// Prüft ob Matrix hermitesch ist: H† = H
pub fn is_hermitian<const N: usize>(matrix: &SMatrix<Complex64, N, N>, tol: f64) -> bool {
    let adjoint = matrix.adjoint();
    let diff = matrix - adjoint;
    frobenius_norm(&diff) < tol
}

/// Normalisiert Vektor
pub fn normalize_vector<const N: usize>(vec: &SVector<Complex64, N>) -> SVector<Complex64, N> {
    let norm = vec.norm();
    if norm < 1e-10 {
        panic!("Cannot normalize zero vector");
    }
    vec.scale(1.0 / norm)
}

/// Berechnet Spur (Trace) einer Matrix
pub fn trace<const N: usize>(matrix: &SMatrix<Complex64, N, N>) -> Complex64 {
    (0..N).map(|i| matrix[(i, i)]).sum()
}

/// Berechnet Determinante (vereinfacht für kleine Matrizen)
pub fn determinant_2x2(matrix: &SMatrix<Complex64, 2, 2>) -> Complex64 {
    matrix[(0, 0)] * matrix[(1, 1)] - matrix[(0, 1)] * matrix[(1, 0)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_matrix_exp_identity() {
        let identity = SMatrix::<f64, 3, 3>::identity();
        let exp_id = matrix_exp_hermitian_3x3(&identity);

        // exp(I) sollte ungefähr e·I sein
        let e = std::f64::consts::E;
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { e } else { 0.0 };
                assert_abs_diff_eq!(exp_id[(i, j)].re, expected, epsilon = 1e-10);
                assert_abs_diff_eq!(exp_id[(i, j)].im, 0.0, epsilon = 1e-10);
            }
        }
    }

    #[test]
    fn test_frobenius_norm() {
        let mut matrix = SMatrix::<Complex64, 2, 2>::zeros();
        matrix[(0, 0)] = Complex64::new(1.0, 0.0);
        matrix[(1, 1)] = Complex64::new(1.0, 0.0);

        let norm = frobenius_norm(&matrix);
        assert_abs_diff_eq!(norm, 2.0_f64.sqrt(), epsilon = 1e-10);
    }

    #[test]
    fn test_is_unitary_identity() {
        let identity = SMatrix::<Complex64, 3, 3>::identity();
        assert!(is_unitary(&identity, 1e-10));
    }

    #[test]
    fn test_is_hermitian() {
        let mut matrix = SMatrix::<Complex64, 2, 2>::zeros();
        matrix[(0, 0)] = Complex64::new(1.0, 0.0);
        matrix[(1, 1)] = Complex64::new(2.0, 0.0);
        matrix[(0, 1)] = Complex64::new(0.0, 1.0);
        matrix[(1, 0)] = Complex64::new(0.0, -1.0);

        assert!(is_hermitian(&matrix, 1e-10));
    }

    #[test]
    fn test_normalize_vector() {
        use nalgebra::SVector;
        let vec = SVector::<Complex64, 3>::new(
            Complex64::new(1.0, 0.0),
            Complex64::new(2.0, 0.0),
            Complex64::new(2.0, 0.0),
        );

        let normalized = normalize_vector(&vec);
        assert_abs_diff_eq!(normalized.norm(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_trace() {
        let mut matrix = SMatrix::<Complex64, 3, 3>::zeros();
        matrix[(0, 0)] = Complex64::new(1.0, 0.0);
        matrix[(1, 1)] = Complex64::new(2.0, 0.0);
        matrix[(2, 2)] = Complex64::new(3.0, 0.0);

        let tr = trace(&matrix);
        assert_abs_diff_eq!(tr.re, 6.0, epsilon = 1e-10);
        assert_abs_diff_eq!(tr.im, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_determinant_2x2() {
        let mut matrix = SMatrix::<Complex64, 2, 2>::zeros();
        matrix[(0, 0)] = Complex64::new(1.0, 0.0);
        matrix[(0, 1)] = Complex64::new(2.0, 0.0);
        matrix[(1, 0)] = Complex64::new(3.0, 0.0);
        matrix[(1, 1)] = Complex64::new(4.0, 0.0);

        let det = determinant_2x2(&matrix);
        // det = 1*4 - 2*3 = -2
        assert_abs_diff_eq!(det.re, -2.0, epsilon = 1e-10);
    }
}
