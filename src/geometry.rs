//! Contains a few structs and methods to faciliate geometry in *n*-dimensional
//! space.

/// A point in *n*-dimensional space.
pub type Point = nalgebra::DVector<Float>;

/// A vector in *n*-dimensional space.
pub type Vector = Point;

/// An *n* by *n* matrix.
pub type Matrix = nalgebra::DMatrix<Float>;

use crate::{Consts, Float};

use approx::abs_diff_eq;

#[derive(Debug)]
/// A hypersphere with a certain center and radius.
pub struct Hypersphere {
    /// The center of the hypersphere.
    pub center: Point,

    /// The squared radius of the hypersphere. We allow negative numbers as a
    /// convenient way to dual + reflect a polytope.
    pub squared_radius: Float,
}

impl Hypersphere {
    /// Constructs a hypersphere with a given dimension and squared radius,
    /// centered at the origin.
    pub fn with_squared_radius(dim: usize, squared_radius: Float) -> Hypersphere {
        Hypersphere {
            center: vec![0.0; dim].into(),
            squared_radius,
        }
    }

    /// Represents the unit hypersphere in a certain number of dimensions.
    pub fn unit(dim: usize) -> Hypersphere {
        Hypersphere::with_squared_radius(dim, 1.0)
    }

    /// Reciprocates a point in place. If it's too close to the sphere's center,
    /// it returns `Err(())` and leaves it unmoved.
    pub fn reciprocate_mut(&self, p: &mut Point) -> Result<(), ()> {
        let mut q = &*p - &self.center;
        let s = q.norm_squared();

        // If any face passes through the dual center, the dual does
        // not exist, and we return early.
        if s < Float::EPS {
            return Err(());
        }

        // Rescales q.
        q /= s;
        q *= self.squared_radius;

        q += &self.center;
        *p = q;

        Ok(())
    }

    /// Reciprocates a point.
    pub fn reciprocate(&self, p: &Point) -> Option<Point> {
        let mut clone = p.clone();
        self.reciprocate_mut(&mut clone).ok().map(|_| clone)
    }

    pub fn random(dim: usize) -> Self {
        use rand::Rng;

        let mut rand = rand::thread_rng();
        let mut center = Point::zeros(dim);
        for coord in center.iter_mut() {
            *coord = rand.gen_range(0.0..=1.0);
        }

        Hypersphere {
            squared_radius: rand.gen_range(0.0..=1.0),
            center,
        }
    }

    /// Returns whether two hyperspheres are "approximately" equal.
    /// Used for testing.
    pub fn approx(&self, sphere: &Hypersphere) -> bool {
        (&self.center - &sphere.center).norm() < Float::EPS
            && self.squared_radius - sphere.squared_radius < Float::EPS
    }
}

/// Represents an (affine) subspace, passing through a given point and generated
/// by a given basis.
///
/// TODO: Use asserts to guarantee that the basis is an orthogonal basis of unit
/// vectors.
pub struct Subspace {
    /// An orthogonal basis for the subspace, defined by unit vectors.
    pub basis: Vec<Vector>,

    /// An "offset", which represents any point on the subspace.
    pub offset: Point,
}

impl Subspace {
    /// Generates a trivial subspace passing through a given point.
    pub fn new(p: Point) -> Self {
        Self {
            basis: Vec::new(),
            offset: p,
        }
    }

    /// Returns the number of dimensions of the ambient space. For the number of
    /// dimensions spanned by the subspace itself, use [`Self::rank`].
    pub fn dim(&self) -> usize {
        self.offset.nrows()
    }

    /// Returns the rank of the subspace, which corresponds to the number of
    /// vectors in its basis.
    pub fn rank(&self) -> usize {
        self.basis.len()
    }

    /// Returns whether the subspace is actually a hyperplane, i.e. a subspace
    /// whose rank is one less than the ambient dimension.
    pub fn is_hyperplane(&self) -> bool {
        self.dim() == self.rank() + 1
    }

    /// Returns whether the subspace is actually of full rank, i.e. a subspace
    /// whose rank equals the ambient dimension.
    pub fn is_full_rank(&self) -> bool {
        self.dim() == self.rank()
    }

    /// Adds a point to the subspace. If the rank increases, returns a new
    /// basis vector for the subspace.
    ///
    /// # Todo:
    /// Implement [this](https://en.wikipedia.org/wiki/Gram%E2%80%93Schmidt_process#Numerical_stability).
    pub fn add(&mut self, p: &Point) -> Option<Point> {
        let mut v = p - self.project(p);

        if v.normalize_mut() > Float::EPS {
            self.basis.push(v.clone());
            Some(v)
        } else {
            None
        }
    }

    /// Creates a subspace from a list of point references.
    pub fn from_point_refs(points: &[&Point]) -> Self {
        let mut points = points.iter();
        let mut h = Self::new(
            (*points
                .next()
                .expect("A hyperplane can't be created from an empty point array!"))
            .clone(),
        );

        for &p in points {
            if h.add(p).is_some() {
                // If the subspace is of full rank, we don't need to check any
                // more points.
                if h.is_full_rank() {
                    return h;
                }
            }
        }

        h
    }

    /// Creates a subspace from a list of points.
    pub fn from_points(points: &[Point]) -> Self {
        Self::from_point_refs(&points.iter().collect::<Vec<_>>())
    }

    /// Projects a point onto the subspace.
    pub fn project(&self, p: &Point) -> Point {
        let p = p - &self.offset;
        let mut q = self.offset.clone();

        for b in &self.basis {
            q += b * p.dot(b);
        }

        q
    }

    /// Calculates the distance from a point to the subspace.
    pub fn distance(&self, p: &Point) -> Float {
        (p - self.project(p)).norm()
    }

    /// Computes a normal vector to the subspace, so that the specified point is
    /// left out of it. Returns `None` if the point given lies on the subspace.
    pub fn normal(&self, p: &Point) -> Option<Vector> {
        (p - self.project(p)).try_normalize(Float::EPS)
    }

    /// Applies a map from the subspace to a lower dimensional space to the
    /// point.
    pub fn flatten(&self, p: &Point) -> Point {
        let p = p - &self.offset;

        Point::from_iterator(self.rank(), self.basis.iter().map(|b| p.dot(b)))
    }

    /// Returns a subspace defined by all points with a given x coordinate.
    pub fn x(rank: usize, x: Float) -> Self {
        // The basis is just all elementary unit vectors save for the
        // (1, 0, ..., 0) one.
        let mut basis = Vec::new();
        for i in 1..rank {
            let mut p = Point::zeros(rank);
            p[i] = 1.0;
            basis.push(p);
        }

        // The offset is the point (x, 0, ..., 0).
        let mut offset = Point::zeros(rank);
        offset[0] = x;

        Self { basis, offset }
    }

    /// Computes a set of independent vectors that span the orthogonal
    /// complement of the subspace.
    pub fn orthogonal_comp(&self) -> Vec<Vector> {
        todo!()
    }
}

/// Represents an (oriented) hyperplane together with a normal vector.
pub struct Hyperplane {
    /// The underlying subspace associated to the hyperplane.
    pub subspace: Subspace,

    /// The normal vector of the hyperplane.
    normal: Vector,
}

impl Hyperplane {
    /// Defines a new oriented hyperplane from a hyperplane and a point outside
    /// of it.
    pub fn new(subspace: Subspace, p: &Point) -> Self {
        debug_assert!(
            subspace.is_hyperplane(),
            "An oriented hyperplane needs to be defined on a hyperplane."
        );

        let normal = subspace
            .normal(p)
            .expect("Specified point not outside the hyperplane.");

        Self { subspace, normal }
    }

    /// Projects a point onto the hyperplane.
    pub fn project(&self, p: &Point) -> Point {
        self.subspace.project(p)
    }

    /// Calculates the signed distance from a point to the hyperplane. Points on
    /// the side of the hyperplane containing the vector have positive distance.
    pub fn distance(&self, p: &Point) -> Float {
        (p - self.project(p)).dot(&self.normal)
    }

    /// Applies a map from the hyperplane to a lower dimensional space to the
    /// point.
    pub fn flatten(&self, p: &Point) -> Point {
        self.subspace.flatten(p)
    }

    /// Returns whether a point is contained on the hyperplane.
    pub fn is_outer(&self, p: &Point) -> bool {
        abs_diff_eq!(self.distance(p), 0.0, epsilon = Float::EPS)
    }

    /// Returns the intersection of itself and a line segment, or `None` if it
    /// doesn't exist.
    pub fn intersect(&self, l: Segment) -> Option<Point> {
        let d0 = self.distance(&l.0);
        let d1 = self.distance(&l.1);
        let t = d1 / (d1 - d0);

        if !(0.0..=1.0).contains(&t) {
            None
        } else {
            Some(l.at(t))
        }
    }

    // Returns a hyperplane defined by all points with a given x coordinate.
    pub fn x(rank: usize, x: Float) -> Self {
        // The normal is the vector (1, 0, ..., 0).
        let mut normal = Vector::zeros(rank);
        normal[0] = 1.0;

        Self {
            subspace: Subspace::x(rank, x),
            normal,
        }
    }
}

/// Represents a line segment between two points.
pub struct Segment(pub Point, pub Point);

impl Segment {
    /// Returns the point at a certain position along the line. If `t` is
    /// between 0 and 1, the point will be contained on the line segment.
    pub fn at(&self, t: Float) -> Point {
        &self.0 * t + &self.1 * (1.0 - t)
    }

    /// Returns the midpoint of the segment.
    pub fn midpoint(&self) -> Point {
        self.at(0.5)
    }
}
