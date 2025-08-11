/// Calculates mine probabilities using iterative scaling
/// Reference:
/// "A simple Minesweeper algorithm", Mike Sheppard, October 9, 2023
/// https://minesweepergame.com/math/a-simple-minesweeper-algorithm-2023.pdf

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

pub fn solve_iterative_scaling<I: AsRef<[usize]>>(
    p: &mut [f64],
    q: &mut [f64],
    omega: &[(usize, I)],
    iterations: usize,
) {
    for _ in 0..iterations {
        for &(n, ref indices) in omega {
            let idx_ref = indices.as_ref();
            let target_p = n as f64;
            let target_q = idx_ref.len() as f64 - target_p;

            scale_vector(p, idx_ref, target_p);
            scale_vector(q, idx_ref, target_q);
        }

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
    fn test_solver_result() {
        let mut p = vec![1.0; 6];
        let mut q = vec![1.0; 6];
        let omega = vec![
            (3, &[0, 1, 2, 3, 4, 5][..]),
            (1, &[1][..]),
            (2, &[0, 1, 2][..]),
            (3, &[0, 1, 2, 3, 4, 5][..]),
        ];

        solve_iterative_scaling(&mut p, &mut q, &omega, 10);

        let expected = vec![0.5, 1.0, 0.5, 0.333333, 0.333333, 0.333333];
        assert!(
            approx_eq_vec(&p, &expected, 1e-3),
            "p = {:?}, expected = {:?}",
            p,
            expected
        );
    }

    #[test]
    fn test_solver_result2() {
        let mut p = vec![1.0; 6];
        let mut q = vec![1.0; 6];
        let omega = vec![
            (3, vec![0, 1, 2, 3, 4, 5]),
            (1, vec![1]),
            (2, vec![0, 1, 2]),
            (3, vec![0, 1, 2, 3, 4, 5]),
        ];

        solve_iterative_scaling(&mut p, &mut q, &omega, 10);

        let expected = vec![0.5, 1.0, 0.5, 0.333333, 0.333333, 0.333333];
        assert!(
            approx_eq_vec(&p, &expected, 1e-3),
            "p = {:?}, expected = {:?}",
            p,
            expected
        );
    }
}
