use minesweeper_rs::solver::{Constraint, solve_iterative_scaling};

fn approx_eq_vec(a: &[f64], b: &[f64], tol: f64) -> bool {
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < tol)
}

fn main() {
    let mut p = vec![1.0; 3];
    let mut q = vec![1.0; 3];

    let constraints = vec![
        Constraint::new(vec![0, 1], 0.79),
        Constraint::new(vec![0, 2], 0.24),
    ];

    let n_it = 50;
    for it in 1..n_it + 1 {
        solve_iterative_scaling(&mut p, &mut q, &constraints, 1);
        if it <= 2 || it == n_it {
            println!("It {it}: P {p:?}");
            println!("It {it}: Q {q:?}");
        }
    }

    let expected_p = vec![0.14985, 0.64015, 0.0901505];
    let expected_q = vec![0.85015, 0.35985, 0.90985];

    assert!(
        approx_eq_vec(&p, &expected_p, 1e-5),
        "p = {p:?}, expected = {expected_p:?}"
    );
    assert!(
        approx_eq_vec(&q, &expected_q, 1e-5),
        "q = {q:?}, expected = {expected_q:?}"
    );
}
