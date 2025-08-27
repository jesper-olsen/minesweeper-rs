// Calculates mine probabilities using iterative scaling
// Reference:
// "A simple Minesweeper algorithm", Mike Sheppard, October 9, 2023
// https://minesweepergame.com/math/a-simple-minesweeper-algorithm-2023.pdf

use crate::Constraint;

const EPS: f64 = 1e-6;

fn scale_vector(vec: &mut [f64], indices: &[usize], target: f64) {
    let sum: f64 = indices.iter().map(|&i| vec[i]).sum();
    if (sum - target).abs() > EPS && sum > EPS {
        let scale = target / sum;
        for &i in indices {
            vec[i] *= scale;
        }
    }
}

pub fn solve_iterative_scaling(
    p: &mut [f64],
    q: &mut [f64],
    constraints: &[Constraint],
    iterations: usize,
) {
    for _ in 0..iterations {
        // Update p's
        for constraint in constraints {
            scale_vector(p, &constraint.cells, constraint.count);
        }
        // Update q's
        for constraint in constraints {
            let target_q = constraint.cells.len() as f64 - constraint.count;
            scale_vector(q, &constraint.cells, target_q);
        }
        // Normalize
        for i in 0..p.len() {
            let total = p[i] + q[i];
            if total > EPS {
                p[i] /= total;
                q[i] /= total;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq_vec(a: &[f64], b: &[f64], tol: f64) -> bool {
        a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < tol)
    }

    #[test]
    fn test_int_example() {
        let mut p = vec![1.0; 6];
        let mut q = vec![1.0; 6];
        let constraints = vec![
            Constraint::new(vec![0, 1, 2, 3, 4, 5], 3),
            Constraint::new(vec![1], 1),
            Constraint::new(vec![0, 1, 2], 2),
            Constraint::new(vec![0, 1, 2, 3, 4, 5], 3),
        ];

        solve_iterative_scaling(&mut p, &mut q, &constraints, 10);

        let expected = vec![0.5, 1.0, 0.5, 0.333333, 0.333333, 0.333333];
        assert!(
            approx_eq_vec(&p, &expected, 1e-3),
            "p = {:?}, expected = {:?}",
            p,
            expected
        );
    }

    #[test]
    fn test_float_example() {
        // From Mike Sheppard's toy example:
        // p1+p2 = 0.79, p1+p3 = 0.24
        let mut p = vec![1.0; 3];
        let mut q = vec![1.0; 3];

        let constraints = vec![
            Constraint::new(vec![0, 1], 0.79),
            Constraint::new(vec![0, 2], 0.24),
        ];

        solve_iterative_scaling(&mut p, &mut q, &constraints, 50);

        let expected_p = vec![0.14985, 0.64015, 0.0901505];
        let expected_q = vec![0.85015, 0.35985, 0.90985];

        assert!(
            approx_eq_vec(&p, &expected_p, 1e-5),
            "p = {:?}, expected = {:?}",
            p,
            expected_p
        );
        assert!(
            approx_eq_vec(&q, &expected_q, 1e-5),
            "q = {:?}, expected = {:?}",
            q,
            expected_q
        );
    }
}
