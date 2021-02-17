use std::f64::consts::PI;
use std::{f64, usize};

use super::Polytope;

pub fn polygon(n: u32, d: u32) -> Polytope {
    let n = n as usize;
    let a = 2.0 * PI / (n as f64) * (d as f64);
    let s = a.sin() * 2.0;

    let mut vertices = Vec::with_capacity(n);
    let mut edges = Vec::with_capacity(n);
    let mut components = vec![Vec::with_capacity(n)];

    for k in 0..n {
        let ka = (k as f64) * a;
        vertices.push(vec![ka.cos() / s, ka.sin() / s].into());
        edges.push(vec![k, (k + 1) % n]);
        components[0].push(k);
    }

    Polytope::new(vertices, vec![edges, components])
}

pub fn tet() -> Polytope {
    let x = 2f64.sqrt() / 4.0;

    let vertices = vec![
        vec![x, x, x].into(),
        vec![-x, -x, x].into(),
        vec![-x, x, -x].into(),
        vec![x, -x, -x].into(),
    ];
    let edges = vec![
        vec![0, 1],
        vec![0, 2],
        vec![0, 3],
        vec![1, 2],
        vec![1, 3],
        vec![2, 3],
    ];
    let faces = vec![vec![0, 1, 3], vec![0, 2, 4], vec![1, 2, 5], vec![3, 4, 5]];
    let components = vec![vec![0, 1, 2, 3]];

    Polytope::new(vertices, vec![edges, faces, components])
}

pub fn cube() -> Polytope {
    let x = 0.5;

    let vertices = vec![
        vec![x, x, x].into(),
        vec![x, x, -x].into(),
        vec![x, -x, -x].into(),
        vec![x, -x, x].into(),
        vec![-x, x, x].into(),
        vec![-x, x, -x].into(),
        vec![-x, -x, -x].into(),
        vec![-x, -x, x].into(),
    ];
    let edges = vec![
        vec![0, 1],
        vec![1, 2],
        vec![2, 3],
        vec![3, 0],
        vec![4, 5],
        vec![5, 6],
        vec![6, 7],
        vec![7, 3],
        vec![0, 4],
        vec![1, 5],
        vec![2, 6],
        vec![3, 7],
    ];
    let faces = vec![
        vec![0, 1, 2, 3],
        vec![4, 5, 6, 7],
        vec![0, 4, 8, 9],
        vec![1, 5, 9, 10],
        vec![2, 6, 10, 11],
        vec![3, 7, 11, 8],
    ];
    let components = vec![vec![0, 1, 2, 3, 4, 5]];

    Polytope::new(vertices, vec![edges, faces, components])
}

pub fn oct() -> Polytope {
    let x = 1.0 / 2f64.sqrt();

    let vertices = vec![
        vec![x, 0.0, 0.0].into(),
        vec![-x, 0.0, 0.0].into(),
        vec![0.0, x, 0.0].into(),
        vec![0.0, 0.0, x].into(),
        vec![0.0, -x, 0.0].into(),
        vec![0.0, 0.0, -x].into(),
    ];
    let edges = vec![
        vec![0, 2],
        vec![0, 3],
        vec![0, 4],
        vec![0, 5],
        vec![1, 2],
        vec![1, 3],
        vec![1, 4],
        vec![1, 5],
        vec![2, 3],
        vec![3, 4],
        vec![4, 5],
        vec![5, 2],
    ];
    let faces = vec![
        vec![0, 1, 8],
        vec![4, 5, 8],
        vec![1, 2, 9],
        vec![5, 6, 9],
        vec![2, 3, 10],
        vec![6, 7, 10],
        vec![3, 0, 11],
        vec![7, 4, 11],
    ];
    let components = vec![vec![0, 1, 2, 3, 4, 5, 6, 7]];

    Polytope::new(vertices, vec![edges, faces, components])
}

/// Creates an [[https://polytope.miraheze.org/wiki/Antiprism | antiprism]] with unit edge length and a given height.
pub fn antiprism_with_height(n: u32, d: u32, mut h: f64) -> Polytope {
    let n = n as usize;
    let a = PI / (n as f64) * (d as f64);
    let s = a.sin() * 2.0;

    let mut vertices = Vec::with_capacity(2 * n);
    let mut edges = Vec::with_capacity(4 * n);
    let mut faces = Vec::with_capacity(2 * n + 2);
    let mut components = vec![Vec::with_capacity(2 * n + 2)];

    for k in 0..(2 * n) {
        // Generates vertices.
        let ka = (k as f64) * a;
        vertices.push(vec![ka.cos() / s, ka.sin() / s, h].into());
        h *= -1.0;

        // Generates edges.
        edges.push(vec![k, (k + 1) % (2 * n)]);
        edges.push(vec![k, (k + 2) % (2 * n)]);

        // Generates faces.
        faces.push(vec![2 * k, 2 * k + 1, (2 * k + 2) % (4 * n)]);

        // Generates component.
        components[0].push(k);
    }

    let (mut base1, mut base2) = (Vec::with_capacity(n), Vec::with_capacity(n));
    for k in 0..n {
        base1.push(4 * k + 1);
        base2.push(4 * k + 3);
    }
    faces.push(base1);
    faces.push(base2);

    components[0].push(2 * n);
    components[0].push(2 * n + 1);

    Polytope::new(vertices, vec![edges, faces, components])
}

pub fn antiprism(n: u32, d: u32) -> Polytope {
    let a = PI / (n as f64);
    let h = (a.cos() - (2.0 * a).cos()).sqrt() / (4.0 * a.sin());

    antiprism_with_height(n, d, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polygon_counts() {
        assert_eq!(polygon(5, 1).el_counts(), vec![5, 5, 1]);
        assert_eq!(polygon(7, 2).el_counts(), vec![7, 7, 1])

        // We aren't implementing polygon compounds yet.
        // assert_eq!(polygon(6, 2).el_counts(), vec![6, 6, 2])
    }

    #[test]
    fn tet_counts() {
        assert_eq!(tet().el_counts(), vec![4, 6, 4, 1])
    }

    #[test]
    fn cube_counts() {
        assert_eq!(cube().el_counts(), vec![8, 12, 6, 1])
    }

    #[test]
    fn oct_counts() {
        assert_eq!(oct().el_counts(), vec![6, 12, 8, 1])
    }

    #[test]
    fn antiprism_counts() {
        assert_eq!(antiprism(5, 1).el_counts(), vec![10, 20, 12, 1]);
        assert_eq!(antiprism(7, 2).el_counts(), vec![14, 28, 16, 1]);

        // We aren't implementing compound antiprisms yet.
        // assert_eq!(antiprism(6, 2).el_counts(), vec![12, 24, 16, 2])
    }
}
