use crate::fri_domain::FriDomain;
use crate::table::processor_table::PROCESSOR_TABLE_PERMUTATION_ARGUMENTS_COUNT;
use crate::table::table_collection::{ExtTableCollection, TableId};
use crate::table::table_column::{
    ExtInstructionTableColumn, ExtJumpStackTableColumn, ExtOpStackTableColumn,
    ExtProcessorTableColumn, ExtRamTableColumn, ExtU32OpTableColumn,
};
use itertools::{izip, Itertools};
use twenty_first::shared_math::mpolynomial::Degree;
use twenty_first::shared_math::traits::PrimeField;
use twenty_first::shared_math::x_field_element::XFieldElement;

pub struct PermArg {
    from_table: TableId,
    from_column: usize,
    to_table: TableId,
    to_column: usize,
}

impl PermArg {
    pub fn new(
        from_table: TableId,
        from_column: usize,
        to_table: TableId,
        to_column: usize,
    ) -> Self {
        PermArg {
            from_table,
            from_column,
            to_table,
            to_column,
        }
    }

    pub fn quotient(
        &self,
        ext_codeword_tables: &ExtTableCollection,
        fri_domain: &FriDomain<XFieldElement>,
    ) -> Vec<XFieldElement> {
        let lhs_codeword = &ext_codeword_tables.data(self.from_table)[self.from_column];
        let rhs_codeword = &ext_codeword_tables.data(self.to_table)[self.to_column];
        let inverse_zerofier = XFieldElement::batch_inversion(
            fri_domain
                .domain_values()
                .into_iter()
                .map(|x| x - 1.into())
                .collect(),
        );

        izip!(lhs_codeword, rhs_codeword, inverse_zerofier)
            .map(|(from, to, z)| (*from - *to) * z)
            .collect_vec()
    }

    pub fn quotient_degree_bound(&self, ext_codeword_tables: &ExtTableCollection) -> Degree {
        ext_codeword_tables.interpolant_degree() - 1

        // todo: check if this is correct. delete before committing if so. (no one will ever see this msg)
        // let lhs_interpolant_degree = ext_codeword_tables.interpolant_degree();
        // let rhs_interpolant_degree = ext_codeword_tables.interpolant_degree();
        // let degree = std::cmp::max(lhs_interpolant_degree, rhs_interpolant_degree);
        //
        // degree - 1
    }

    pub fn evaluate_difference(&self, points: &[Vec<XFieldElement>]) -> XFieldElement {
        let lhs = points[self.from_table as usize][self.from_column];
        let rhs = points[self.to_table as usize][self.to_column];

        lhs - rhs
    }
}

impl PermArg {
    /// A Permutation Argument between Processor Table and Instruction Table.
    pub fn processor_instruction_perm_arg() -> Self {
        PermArg::new(
            TableId::ProcessorTable,
            ExtProcessorTableColumn::InstructionTablePermArg.into(),
            TableId::InstructionTable,
            ExtInstructionTableColumn::RunningProductPermArg.into(),
        )
    }

    /// A Permutation Argument between Processor Table and Jump-Stack Table.
    pub fn processor_jump_stack_perm_arg() -> Self {
        PermArg::new(
            TableId::ProcessorTable,
            ExtProcessorTableColumn::JumpStackTablePermArg.into(),
            TableId::JumpStackTable,
            ExtJumpStackTableColumn::RunningProductPermArg.into(),
        )
    }

    /// A Permutation Argument between Processor Table and Op-Stack Table.
    pub fn processor_op_stack_perm_arg() -> Self {
        PermArg::new(
            TableId::ProcessorTable,
            ExtProcessorTableColumn::OpStackTablePermArg.into(),
            TableId::OpStackTable,
            ExtOpStackTableColumn::RunningProductPermArg.into(),
        )
    }

    /// A Permutation Argument between Processor Table and RAM Table.
    pub fn processor_ram_perm_arg() -> Self {
        PermArg::new(
            TableId::ProcessorTable,
            ExtProcessorTableColumn::RamTablePermArg.into(),
            TableId::RamTable,
            ExtRamTableColumn::RunningProductPermArg.into(),
        )
    }

    /// A Permutation Argument with the u32 Op-Table.
    pub fn processor_u32_perm_arg() -> Self {
        PermArg::new(
            TableId::ProcessorTable,
            ExtProcessorTableColumn::U32OpTablePermArg.into(),
            TableId::U32OpTable,
            ExtU32OpTableColumn::RunningProductPermArg.into(),
        )
    }

    // FIXME: PROCESSOR_TABLE_PERMUTATION_ARGUMENTS_COUNT is incidentally ALL permutation arguments; create new constant?
    pub fn all_permutation_arguments() -> [Self; PROCESSOR_TABLE_PERMUTATION_ARGUMENTS_COUNT] {
        [
            Self::processor_instruction_perm_arg(),
            Self::processor_jump_stack_perm_arg(),
            Self::processor_op_stack_perm_arg(),
            Self::processor_ram_perm_arg(),
            Self::processor_u32_perm_arg(),
        ]
    }
}

#[cfg(test)]
mod permutation_argument_tests {
    use super::*;

    #[test]
    fn all_permutation_arguments_link_from_processor_table_test() {
        for perm_arg in PermArg::all_permutation_arguments() {
            assert_eq!(TableId::ProcessorTable, perm_arg.from_table);
        }
    }
}
