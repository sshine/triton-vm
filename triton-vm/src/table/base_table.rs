use super::super::fri_domain::FriDomain;
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::ops::Range;
use twenty_first::shared_math::b_field_element::BFieldElement;
use twenty_first::shared_math::mpolynomial::{Degree, MPolynomial};
// use twenty_first::shared_math::other::{is_power_of_two, roundup_npo2};
use twenty_first::shared_math::polynomial::Polynomial;
use twenty_first::shared_math::traits::{GetRandomElements, PrimeField};
use twenty_first::shared_math::x_field_element::XFieldElement;

type BWord = BFieldElement;
type XWord = XFieldElement;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BaseTable<FieldElement: PrimeField> {
    /// The width of each `data` row in the base version of the table
    base_width: usize,

    /// The width of each `data` row in the extended version of the table
    full_width: usize,

    /// The table data (trace data). Represents every intermediate
    matrix: Vec<Vec<FieldElement>>,

    /// The name of the table. Mostly for debugging purpose.
    pub(crate) name: String,

    /// AIR constraints, to be populated upon extension
    pub(crate) boundary_constraints: Option<Vec<MPolynomial<FieldElement>>>,
    pub(crate) transition_constraints: Option<Vec<MPolynomial<FieldElement>>>,
    pub(crate) consistency_constraints: Option<Vec<MPolynomial<FieldElement>>>,
    pub(crate) terminal_constraints: Option<Vec<MPolynomial<FieldElement>>>,

    /// quotient degrees, to be populated upon extension
    pub(crate) boundary_quotient_degree_bounds: Option<Vec<i64>>,
    pub(crate) transition_quotient_degree_bounds: Option<Vec<i64>>,
    pub(crate) consistency_quotient_degree_bounds: Option<Vec<i64>>,
    pub(crate) terminal_quotient_degree_bounds: Option<Vec<i64>>,
}

#[allow(clippy::too_many_arguments)]
impl<DataPF: PrimeField> BaseTable<DataPF> {
    pub fn new(
        base_width: usize,
        full_width: usize,
        matrix: Vec<Vec<DataPF>>,
        name: String,
    ) -> Self {
        BaseTable {
            base_width,
            full_width,
            matrix,
            name,
            boundary_constraints: None,
            transition_constraints: None,
            consistency_constraints: None,
            terminal_constraints: None,
            boundary_quotient_degree_bounds: None,
            transition_quotient_degree_bounds: None,
            consistency_quotient_degree_bounds: None,
            terminal_quotient_degree_bounds: None,
        }
    }

    pub fn extension(
        base_table: BaseTable<DataPF>,
        interpolant_degree: Degree,
        boundary_constraints: Vec<MPolynomial<DataPF>>,
        transition_constraints: Vec<MPolynomial<DataPF>>,
        consistency_constraints: Vec<MPolynomial<DataPF>>,
        terminal_constraints: Vec<MPolynomial<DataPF>>,
    ) -> Self {
        let full_width = base_table.full_width;

        let boundary_quotient_degree_bounds =
            Self::compute_degree_bounds(&boundary_constraints, interpolant_degree, full_width);
        let transition_quotient_degree_bounds = Self::compute_degree_bounds(
            &transition_constraints,
            interpolant_degree,
            2 * full_width,
        );
        let consistency_quotient_degree_bounds =
            Self::compute_degree_bounds(&consistency_constraints, interpolant_degree, full_width);
        let terminal_quotient_degree_bounds =
            Self::compute_degree_bounds(&terminal_constraints, interpolant_degree, full_width);

        BaseTable {
            boundary_constraints: Some(boundary_constraints),
            transition_constraints: Some(transition_constraints),
            consistency_constraints: Some(consistency_constraints),
            terminal_constraints: Some(terminal_constraints),
            boundary_quotient_degree_bounds: Some(boundary_quotient_degree_bounds),
            transition_quotient_degree_bounds: Some(transition_quotient_degree_bounds),
            consistency_quotient_degree_bounds: Some(consistency_quotient_degree_bounds),
            terminal_quotient_degree_bounds: Some(terminal_quotient_degree_bounds),
            ..base_table
        }
    }

    /// Computes the degree bounds of the quotients given the AIR constraints and the interpolant
    /// degree. The AIR constraints are defined over a symbolic ring with `full_width`-many
    /// variables.
    fn compute_degree_bounds(
        air_constraints: &[MPolynomial<DataPF>],
        interpolant_degree: Degree,
        full_width: usize,
    ) -> Vec<Degree> {
        air_constraints
            .iter()
            .map(|mpo| mpo.symbolic_degree_bound(&vec![interpolant_degree; full_width]) - 1)
            .collect()
    }

    /// Create a `BaseTable<DataPF>` with the same parameters, but new `matrix` data.
    pub fn with_data(&self, matrix: Vec<Vec<DataPF>>) -> Self {
        BaseTable {
            matrix,
            name: format!("{} with data", self.name),
            ..self.to_owned()
        }
    }
}

/// Create a `BaseTable<XWord` from a `BaseTable<BWord>` with the same parameters lifted from the
/// B-Field into the X-Field (where applicable), but new `matrix` data.
impl BaseTable<BWord> {
    pub fn with_lifted_data(
        &self,
        matrix: Vec<Vec<XWord>>,
        num_trace_randomizers: usize,
    ) -> BaseTable<XWord> {
        BaseTable::new(
            self.base_width,
            self.full_width,
            matrix,
            format!("{} with lifted data", self.name),
        )
    }
}

pub trait HasBaseTable<DataPF: PrimeField> {
    fn to_base(&self) -> &BaseTable<DataPF>;
    fn to_mut_base(&mut self) -> &mut BaseTable<DataPF>;

    fn base_width(&self) -> usize {
        self.to_base().base_width
    }

    fn full_width(&self) -> usize {
        self.to_base().full_width
    }

    fn data(&self) -> &Vec<Vec<DataPF>> {
        &self.to_base().matrix
    }

    fn mut_data(&mut self) -> &mut Vec<Vec<DataPF>> {
        &mut self.to_mut_base().matrix
    }
}

fn disjoint_domain<DataPF: PrimeField>(
    domain_length: usize,
    disjoint_domain: &[DataPF],
    ring_one: DataPF,
) -> Vec<DataPF> {
    // Why do we still have this? 😩
    let zero = ring_one.ring_zero();
    (0..2_usize.pow(32))
        .map(|d| zero.new_from_usize(d))
        .filter(|d| !disjoint_domain.contains(d))
        .take(domain_length)
        .collect_vec()
}

pub trait BaseTableTrait<DataPF>: HasBaseTable<DataPF>
where
    // Self: Sized,
    DataPF: PrimeField + GetRandomElements,
{
    // Abstract functions that individual structs implement

    fn get_padding_row(&self) -> Vec<DataPF>;

    // Generic functions common to all tables

    fn name(&self) -> String {
        self.to_base().name.clone()
    }

    /// Add padding to a table so that its height becomes the same as other tables.
    ///
    /// Use table-specific padding via `.get_padding_row()`.
    fn pad(&mut self, shared_padded_height: usize) {
        while self.data().len() != shared_padded_height {
            let padding_row = self.get_padding_row();
            self.mut_data().push(padding_row);
        }
    }

    fn low_degree_extension(
        &self,
        fri_domain: &FriDomain<DataPF>,
        omicron: DataPF,
        shared_padded_height: usize,
        num_trace_randomizers: usize,
        columns: Range<usize>,
    ) -> Vec<Vec<DataPF>> {
        // FIXME: Table<> supports Vec<[DataPF; WIDTH]>, but FriDomain does not (yet).
        self.interpolate_columns(
            fri_domain,
            omicron,
            shared_padded_height,
            num_trace_randomizers,
            columns,
        )
        .par_iter()
        .map(|polynomial| fri_domain.evaluate(polynomial))
        .collect()
    }

    /// Return the interpolation of columns. The `column_indices` variable
    /// must be called with *all* the column indices for this particular table,
    /// if it is called with a subset, it *will* fail.
    fn interpolate_columns(
        &self,
        fri_domain: &FriDomain<DataPF>,
        omicron: DataPF,
        shared_padded_height: usize,
        num_trace_randomizers: usize,
        columns: Range<usize>,
    ) -> Vec<Polynomial<DataPF>> {
        // FIXME: Inject `rng` instead.
        let mut rng = rand::thread_rng();

        // Ensure that `matrix` is set and padded before running this function
        assert_eq!(
            shared_padded_height,
            self.data().len(),
            "{}: Table data must be padded before interpolation",
            self.name()
        );

        if shared_padded_height == 0 {
            return vec![Polynomial::ring_zero(); columns.len()];
        }

        // FIXME: Unfold with multiplication instead of mapping with power.
        let omicron_domain = (0..shared_padded_height)
            .map(|i| omicron.mod_pow_u32(i as u32))
            .collect_vec();

        let one = fri_domain.omega.ring_one();
        let randomizer_domain = disjoint_domain(num_trace_randomizers, &omicron_domain, one);

        let interpolation_domain = vec![omicron_domain, randomizer_domain].concat();
        let mut all_randomized_traces = vec![];
        let data = self.data();

        for col in columns {
            let trace = data.iter().map(|row| row[col]).collect();
            let randomizers = DataPF::random_elements(num_trace_randomizers, &mut rng);
            let randomized_trace = vec![trace, randomizers].concat();
            assert_eq!(
                randomized_trace.len(),
                interpolation_domain.len(),
                "Length of x values and y values must match"
            );
            all_randomized_traces.push(randomized_trace);
        }

        all_randomized_traces
            .par_iter()
            .map(|randomized_trace| {
                Polynomial::fast_interpolate(
                    &interpolation_domain,
                    randomized_trace,
                    &fri_domain.omega,
                    fri_domain.length,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod test_base_table {
    use crate::table::base_table::disjoint_domain;
    use twenty_first::shared_math::b_field_element::BFieldElement;

    #[test]
    fn disjoint_domain_test() {
        let one = BFieldElement::ring_one();
        let domain = [2.into(), 5.into(), 4.into()];
        let ddomain = disjoint_domain(5, &domain, one);
        assert_eq!(vec![0.into(), one, 3.into(), 6.into(), 7.into()], ddomain);
    }
}
