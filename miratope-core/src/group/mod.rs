//! Contains methods to generate many symmetry groups.

pub mod cyclic;
pub mod gen_iter;
pub mod group_item;
pub mod pairs;
pub mod permutation;

pub use gen_iter::*;

use std::{
    array, iter,
    iter::{Filter, Map, Once},
    vec,
};

use crate::{
    cox::{cd::CdResult, Cox},
    geometry::Matrix,
    Float,
};

use nalgebra::{
    allocator::Allocator, dmatrix, Const, DefaultAllocator, Quaternion, Rotation, UnitQuaternion,
};

use self::{
    cyclic::Cyclic,
    group_item::GroupItem,
    pairs::{AsPair, PairMap},
    permutation::{PermutationIter, SPermutation},
};

/// The type of the dimension associated to an iterator.
type Dim<I> = <<I as Iterator>::Item as GroupItem>::Dim;

/// An iterator over the elements of a group.
///
/// # Safety
/// By creating a value of this type, you're asserting a series of various
/// complex conditions:
/// 1. The elements of your iterator are closed under some operation `*`.
/// 2. This operation `*` is associative on the elements of your iterator.
/// 3. The iterator contains an identity element for `*`.
/// 4. Every element has an inverse in the iterator.
#[derive(Clone)]
pub struct Group<I: Iterator>
where
    I::Item: GroupItem,
{
    dim: Dim<I>,
    iter: I,
}

impl<I: Iterator> Iterator for Group<I>
where
    I::Item: GroupItem,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<I: Iterator> Group<I>
where
    I::Item: GroupItem,
{
    /// An iterator over the elements of a group.
    ///
    /// # Safety
    /// todo: write group axioms and such
    pub unsafe fn new(dim: Dim<I>, iter: I) -> Self {
        Self { dim, iter }
    }

    /// Builds an isomorphism between groups. This behaves as a wrapper for
    /// [`iter::Map`].
    ///
    /// # Safety
    /// The user must verify that this map is indeed an isomorphism. That is,
    /// multiplication in one type must directly correspond to multiplication
    /// in the other.
    pub unsafe fn iso<T: GroupItem, F: FnMut(I::Item) -> T>(
        self,
        dim: T::Dim,
        f: F,
    ) -> Group<Map<I, F>> {
        Group::new(dim, self.iter.map(f))
    }

    /// Builds a subgroup by taking all elements satisfying a certain predicate.
    ///
    /// # Safety
    /// The user must verify that the specified elements indeed form a group.
    pub unsafe fn sub<F: FnMut(&I::Item) -> bool>(self, f: F) -> Group<Filter<I, F>> {
        Group::new(self.dim, self.iter.filter(f))
    }

    /// Gets all elements of `self` and stores them in a cache, so that they can
    /// be quickly iterated over again.
    pub fn cache(self) -> Group<vec::IntoIter<I::Item>> {
        // Safety: cacheing a group does not change any of its algebraic properties.
        unsafe { Group::new(self.dim, self.collect::<Vec<_>>().into_iter()) }
    }

    /// Returns the permutation group associated to `self`.
    pub fn permutations<D: nalgebra::Dim>(self) -> Group<PermutationIter<I::Item, D>>
    where
        DefaultAllocator: Allocator<usize, D> + Allocator<I::Item, D>,
        I::Item: Clone,
    {
        let vec: Vec<_> = self.collect();
        unsafe { Group::new(D::from_usize(vec.len()), PermutationIter::new(vec)) }
    }
}

impl<T: GroupItem> Group<Once<T>> {
    /// Builds the group containing only the identity.
    pub fn trivial(dim: T::Dim) -> Self {
        // Safety: the identity always forms a group.
        unsafe { Self::new(dim, iter::once(T::id(dim))) }
    }
}

impl<T: GroupItem + Clone> Group<Cyclic<T>> {
    /// Builds a cyclic group from a single generator.
    pub fn cyclic_gen(dim: T::Dim, gen: T) -> Self {
        // Safety: this construction always yields a group.
        unsafe { Self::new(dim, Cyclic::new(gen)) }
    }
}

impl<T: Float> Group<Cyclic<Matrix<T>>> {
    /// Builds the cyclic group generated by a 2π / n rotation.
    pub fn cyclic(n: u32) -> Self {
        let (s, c) = (T::TAU / T::u32(n)).fsin_cos();
        Self::cyclic_gen(2, dmatrix![c, -s; s, c])
    }
}

impl<T: Float> Group<GenIter<Matrix<T>>> {
    /// Parses a diagram and turns it into a Coxeter group.
    pub fn parse(input: &str) -> CdResult<Option<Self>> {
        GenIter::parse(input).map(|gens| gens.map(Into::into))
    }

    /// Parses a diagram and turns it into a Coxeter group.
    pub fn parse_unwrap(input: &str) -> Self {
        Self::parse(input).unwrap().unwrap()
    }

    /// Returns the A(n) group.
    pub fn simplex(n: usize) -> Self {
        Cox::a(n).group().unwrap()
    }

    /// Returns the B(n) group.
    pub fn hypercube(n: usize) -> Self {
        Cox::b(n).group().unwrap()
    }

    /// Returns the D(n) group.
    pub fn demihypercube(n: usize) -> Self {
        Cox::d(n).group().unwrap()
    }

    /// Returns the E(n) group.
    pub fn gosset(n: usize) -> Self {
        assert!((4..=8).contains(&n));
        Cox::e(n).group().unwrap()
    }

    /// Returns the H(n) group.
    pub fn pentagonal(n: usize) -> Self {
        assert!((2..=4).contains(&n));
        Cox::h(n).group().unwrap()
    }
}

impl<T: GroupItem> Group<array::IntoIter<T, 2>> {
    /// Creates a group with two elements: the identity and the specified
    /// generator.
    ///
    /// # Safety
    /// The generator must be an involution.
    pub unsafe fn two(dim: T::Dim, gen: T) -> Self {
        Self::new(dim, array::IntoIter::new([T::id(dim), gen]))
    }
}

impl<T: Float> Group<array::IntoIter<Matrix<T>, 2>> {
    /// Builds the group containing central inversion only.
    pub fn central_inv(dim: usize) -> Self {
        assert!(dim >= 1);
        // Safety: (-I)^2 = I.
        unsafe { Self::two(dim, -Matrix::identity(dim, dim)) }
    }

    /// Builds the group containing reflection at a given coordinate only.
    pub fn reflection_at(dim: usize, idx: usize) -> Self {
        assert!(dim >= 1);
        let mut refl = Matrix::identity(dim, dim);
        refl[(idx, idx)] = -T::ONE;
        unsafe { Self::two(dim, refl) }
    }
}

impl Group<array::IntoIter<SPermutation<2>, 2>> {
    /// Returns the symmetric group of order 2.
    pub fn involution() -> Self {
        // Safety: this is a valid permutation, and also an involution.
        unsafe { Self::two(Const::<2>, SPermutation::from_data([1, 0])) }
    }
}

type MatrixProductIter<T> = PairMap<
    (Vec<Matrix<T>>, Vec<Matrix<T>>),
    for<'a, 'b> fn(&'a Matrix<T>, &'b Matrix<T>) -> Matrix<T>,
>;

impl<T: Float> Group<MatrixProductIter<T>> {
    /// Builds the dihedral group generated by a 2π / n rotation and reflection
    /// by the x axis.
    pub fn dihedral_2(n: u32) -> Self {
        // Safety: Conjugation doesn't commute with the cyclic group, but
        // appending it still forms a valid group.
        unsafe { Group::cyclic(n).with_reflection_at(0) }
    }

    /// Builds the dihedral group generated by a 2π / n rotation and reflection
    /// by the x axis.
    pub fn dihedral_3(n: u32) -> Self {
        // Safety: Conjugation doesn't commute with the cyclic group, but
        // appending it still forms a valid group.
        unsafe { Group::cyclic(n).pad(1).with_reflection_at(2) }
    }
}

/// An iterator over the elements of a matrix group.
impl<T: Float, I: Iterator<Item = Matrix<T>>> Group<I> {
    /// Pads all elements of `self` by adding the specified amount of
    /// dimensions to each matrix.
    pub fn pad(self, dim: usize) -> Group<impl Iterator<Item = Matrix<T>>> {
        self.direct_product(Group::trivial(dim))
    }

    /// Buils the rotation subgroup of a group.
    pub fn rotations(self) -> Group<impl Iterator<Item = Matrix<T>>> {
        // Safety: matrices with determinant 1 are closed under multiplication
        // and inverses.
        //
        // The determinant might not be exactly 1, so we're extra lenient and
        // just test for positive determinants, which are likewise closed under
        // multiplications and inverses.
        unsafe { self.sub(|el| el.determinant() > T::ZERO) }
    }

    /// Returns the group determined by all products between elements of the
    /// first and the second group.
    ///
    /// # Safety
    /// The user must make sure the groups commute with one another, or the
    /// result will not necessarily be a group.
    pub unsafe fn matrix_product<J: Iterator<Item = Matrix<T>>>(
        self,
        g: Group<J>,
    ) -> Group<MatrixProductIter<T>> {
        Group::new(
            self.dim,
            (self.collect::<Vec<_>>(), g.collect::<Vec<_>>())
                .into_pairs()
                .map(|a, b| a * b),
        )
    }

    /// Returns the specified group with central inversion appended to all
    /// elements.
    ///
    /// # Safety
    /// The group must not contain central inversion already.
    pub unsafe fn with_central_inv(self) -> Group<impl Iterator<Item = Matrix<T>>> {
        let dim = self.dim;
        self.matrix_product(Group::central_inv(dim))
    }

    /// Returns the specified group with reflection at a given coordinate
    /// appended to all elements.
    ///
    /// # Safety
    /// This must form a valid group.
    pub unsafe fn with_reflection_at(self, idx: usize) -> Group<MatrixProductIter<T>> {
        let dim = self.dim;
        self.matrix_product(Group::reflection_at(dim, idx))
    }

    /// Calculates the direct product of two groups. Pairs of matrices are then
    /// mapped to their direct sum.
    pub fn direct_product<J: Iterator<Item = Matrix<T>>>(
        self,
        g: Group<J>,
    ) -> Group<impl Iterator<Item = Matrix<T>>> {
        // Safety: the direct sum is always a group isomorphic to the Cartesian product.
        unsafe {
            Group::new(
                self.dim + g.dim,
                (self.collect::<Vec<_>>(), g.collect::<Vec<_>>())
                    .into_pairs()
                    .map(direct_sum),
            )
        }
    }

    /// Builds a swirlchoron group. This is the diploid group construction from
    /// "On Quaternions and Octonions" by John H. Conway and Derek A. Smith.
    ///
    /// This method allows to specify a homomorphism between both rotation
    /// groups and some abstract group.
    ///
    /// # Safety
    /// Both groups must be rotation groups, and the passed functions must be
    /// group homomorphisms.
    pub unsafe fn swirl_hom<
        U: GroupItem + 'static, //why static?
        J: Iterator<Item = Matrix<T>>,
        A: FnMut(&Matrix<T>) -> U,
        B: FnMut(&Matrix<T>) -> U,
    >(
        self,
        g: Group<J>,
        mut alpha: A,
        mut beta: B,
    ) -> Group<impl Iterator<Item = Matrix<T>>> {
        assert_eq!(self.dim, 3);
        assert_eq!(g.dim, 3);

        Group::new(
            4,
            (
                self.map(|mat| (alpha(&mat), mat_to_quat(&mat)))
                    .collect::<Vec<_>>(),
                g.map(|mat| (beta(&mat), mat_to_quat(&mat)))
                    .collect::<Vec<_>>(),
            )
                .into_pairs()
                .filter_map(|(alpha, q), (beta, r)| {
                    (alpha.eq(beta)).then(|| {
                        let prod = mat_from_quats(q.quaternion(), r.quaternion());
                        array::IntoIter::new([-&prod, prod])
                    })
                })
                .flatten(),
        )
    }

    /// Builds a swirlchoron group. This is the diploid group construction from
    /// "On Quaternions and Octonions" by John H. Conway and Derek A. Smith.
    ///
    /// # Safety
    /// Both groups must be rotation groups.
    pub unsafe fn swirl<J: Iterator<Item = Matrix<T>>>(
        self,
        g: Group<J>,
    ) -> Group<impl Iterator<Item = Matrix<T>>> {
        self.swirl_hom(g, |_| (), |_| ())
    }

    /// Generates a step prism group from a base group and a homomorphism into
    /// another group.
    ///
    /// # Safety
    /// The specified function must be a group homomorphism.
    pub unsafe fn step_hom<F: FnMut(&Matrix<T>) -> Matrix<T>>(
        self,
        mut f: F,
    ) -> Group<impl Iterator<Item = Matrix<T>>> {
        let dim = self.dim;
        self.iso(2 * dim, move |mat| direct_sum(&mat, &f(&mat)))
    }

    /*
    /// Generates the [wreath product](https://en.wikipedia.org/wiki/Wreath_product)
    /// of two symmetry groups.
    pub fn wreath<D: nalgebra::Dim, J: Iterator<Item = Permutation<D>>>(self, h: Group<J>) -> Self
    where
        J::Item: GroupItem,
    {
        let h = h.elements();
        let g_dim = g.dim;
        let dim = g_dim * h.len();

        // Indexes each element in h.
        let mut h_indices = BTreeMap::new();

        for (i, h_el) in h.iter().enumerate() {
            h_indices.insert(MatrixOrd::new(h_el.clone()), i);
        }

        // Converts h into a permutation group.
        let mut permutations = Vec::with_capacity(h.len());

        for h_el_1 in &h {
            let mut perm = Vec::with_capacity(h.len());

            for h_el_2 in &h {
                perm.push(
                    *h_indices
                        .get(&MatrixOrd::new(h_el_1 * h_el_2))
                        .expect("h is not a valid group!"),
                );
            }

            permutations.push(perm);
        }

        // Computes the direct product of g with itself |h| times.
        let g_prod = vec![&g; h.len() - 1]
            .into_iter()
            .cloned()
            .fold(g.clone(), Group::direct_product);

        Self::new(
            dim,
            g_prod
                .map(move |g_el| {
                    let mut matrices = Vec::new();

                    for perm in &permutations {
                        let mut new_el = Matrix::zeros(dim, dim);

                        // Permutes the blocks on the diagonal of g_el.
                        for (i, &j) in perm.iter().enumerate() {
                            for x in 0..g_dim {
                                for y in 0..g_dim {
                                    new_el[(i * g_dim + x, j * g_dim + y)] =
                                        g_el[(i * g_dim + x, i * g_dim + y)];
                                }
                            }
                        }

                        matrices.push(new_el);
                    }

                    matrices.into_iter()
                })
                .flatten(),
        )
    }
    */
}

fn mat_to_quat<T: Float>(mat: &Matrix<T>) -> UnitQuaternion<T> {
    UnitQuaternion::from_rotation_matrix(&Rotation::from_matrix_unchecked(
        mat.fixed_slice::<3, 3>(0, 0).into(),
    ))
}

/// Converts a quaternion into a matrix, depending on whether it's a left or
/// right quaternion multiplication.
fn mat_from_quats<T: Float>(q: &Quaternion<T>, r: &Quaternion<T>) -> Matrix<T> {
    Matrix::from_iterator(
        4,
        4,
        // q, q * i, q * j, q * k.
        array::IntoIter::new([
            *q,
            [q.w, q.k, -q.j, -q.i].into(),
            [-q.k, q.w, q.i, -q.j].into(),
            [q.j, -q.i, q.w, -q.k].into(),
        ])
        .map(|q| {
            let arr = (q * r).coords.data.0[0];
            array::IntoIter::new([arr[3], arr[0], arr[1], arr[2]])
        })
        .flatten(),
    )
}

/// Computes the [direct sum](https://en.wikipedia.org/wiki/Block_matrix#Direct_sum)
/// of two matrices.
fn direct_sum<T: Float>(mat1: &Matrix<T>, mat2: &Matrix<T>) -> Matrix<T> {
    let dim1 = mat1.nrows();
    let dim = dim1 + mat2.nrows();

    Matrix::from_fn(dim, dim, |i, j| {
        if i < dim1 {
            if j < dim1 {
                mat1[(i, j)]
            } else {
                T::ZERO
            }
        } else if j >= dim1 {
            mat2[(i - dim1, j - dim1)]
        } else {
            T::ZERO
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gcd::Gcd;

    /// Tests a given symmetry group.
    fn test<I: Iterator<Item = Matrix<f32>>>(
        group: Group<I>,
        order: usize,
        rot_order: usize,
        name: &str,
    ) {
        // Makes testing multiple derived groups faster.
        let group = group.cache();

        // Tests the order of the group.
        assert_eq!(
            group.clone().count(),
            order,
            "{} does not have the expected order.",
            name
        );

        // Tests the order of the rotational subgroup.
        assert_eq!(
            group.rotations().count(),
            rot_order,
            "The rotational group of {} does not have the expected order.",
            name
        );
    }

    /// Tests the trivial group in various dimensions.
    #[test]
    fn i() {
        for n in 1..=10 {
            test(Group::trivial(n), 1, 1, &format!("I^{}", n))
        }
    }

    /// Tests the group consisting of the identity and a central inversion in
    /// various dimensions.
    #[test]
    fn pm_i() {
        for n in 1..=10 {
            test(
                Group::central_inv(n),
                2,
                (n + 1) % 2 + 1,
                &format!("±I{}", n),
            )
        }
    }

    /// Tests the I2(*n*) symmetries, which correspond to the symmetries of a
    /// regular *n*-gon.    
    #[test]
    fn dihedral() {
        for n in 2..=10 {
            for d in 1..n {
                if n.gcd(d) != 1 {
                    continue;
                }

                let n_usize = n as usize;

                test(
                    Group::dihedral_2(n),
                    2 * n_usize,
                    n_usize,
                    &format!("I2({})", n),
                );
            }
        }
    }

    /// Tests the A3⁺ @ (I2(*n*) × I) symmetries, the tetrahedron swirl
    /// symmetries.
    #[test]
    fn a3_p_swirl_i2xi_p() {
        for n in 2..10 {
            let order = 24 * (n as usize);

            test(
                unsafe { Group::simplex(3).rotations().swirl(Group::cyclic(n).pad(1)) },
                order,
                order,
                &format!("A3⁺ @ (I2({}) × I)", n),
            )
        }
    }

    /// Tests the A*n* symmetries, which correspond to the symmetries of the
    /// regular simplices.
    #[test]
    fn simplex() {
        let mut order = 2;

        for n in 2..=6 {
            order *= n + 1;

            test(Group::simplex(n), order, order / 2, &format!("A{}", n))
        }
    }
    /// Tests the ±A*n* symmetries, which correspond to the symmetries of the
    /// compound of two simplices.
    #[test]
    fn pm_an() {
        let mut order = 4;

        for n in 2..=6 {
            order *= n + 1;

            test(
                unsafe { Group::matrix_product(Group::simplex(n), Group::central_inv(n)) }, //change
                order,
                order / 2,
                &format!("±A{}", n),
            )
        }
    }

    /// Tests the BC*n* symmetries, which correspond to the symmetries of the
    /// regular hypercube and orthoplex.
    #[test]
    fn hypercube() {
        let mut order = 2;

        for n in 2..=6 {
            order *= n * 2;
            test(Group::hypercube(n), order, order / 2, &format!("BC{}", n))
        }
    }

    /// Tests the H*n* symmetries, which correspond to the symmetries of a
    /// regular dodecahedron and a regular hecatonicosachoron.
    #[test]
    fn h() {
        test(Group::parse_unwrap("o5o3o"), 120, 60, "H3");
        test(Group::parse_unwrap("o5o3o3o"), 14400, 7200, "H4");
    }

    /// Tests the E6 symmetry group.
    #[test]
    fn e6() {
        test(Group::parse_unwrap("o3o3o3o3o *c3o"), 51840, 25920, "E6");
    }

    #[test]
    fn pairs() {
        assert_eq!(
            (vec![1, 2], vec![3, 4])
                .into_pairs()
                .cloned()
                .collect::<Vec<_>>(),
            vec![(1, 3), (2, 3), (1, 4), (2, 4)]
        );
    }

    /// Tests the E7 symmetry group. This is very expensive, so we enable it
    /// only on release mode.
    #[test]
    #[cfg(not(debug_assertions))]
    fn e7() {
        test(
            Group::parse_unwrap("o3o3o3o3o3o *c3o"),
            2903040,
            1451520,
            "E7",
        );
    }

    #[test]
    /// Tests the direct product of A3 with itself.
    fn a3xa3() {
        let a3 = Group::parse_unwrap("o3o3o");
        let g = Group::direct_product(a3.clone(), a3);
        test(g, 576, 288, "A3×A3");
    }

    /* #[test]
    /// Tests the wreath product of A3 with A1.
    fn a3_wr_a1() {
        test(
            Group::wreath(Group::a(3), Group::a(1)),
            1152,
            576,
            "A3 ≀ A1",
        );
    }*/

    #[test]
    /// Tests out some step prisms.
    fn step() {
        for n in 1..10 {
            for d in 1..n {
                let n_usize = n as usize;

                test(
                    unsafe { Group::step_hom(Group::cyclic(n), move |mat| mat.pow(d).unwrap()) }, //change
                    n_usize,
                    n_usize,
                    "Step prismatic n-d",
                );
            }
        }
    }
}
