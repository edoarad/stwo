use num_traits::One;

use super::component::{Input, N_COLUMNS};
use crate::core::fields::m31::BaseField;
use crate::core::fields::FieldExpOps;

// TODO(ShaharS), try to make it into a for loop and use intermiddiate variables to save
// computation.
/// Given a private input, write the trace row for the wide Fibonacci example to dst.
pub fn write_trace_row(
    dst: &mut [Vec<BaseField>],
    private_input: &Input,
    row_index: usize,
) -> (BaseField, BaseField) {
    dst[0][row_index] = private_input.a;
    dst[1][row_index] = private_input.b;
    for i in 2..N_COLUMNS {
        dst[i][row_index] = dst[i - 1][row_index].square() + dst[i - 2][row_index].square();
    }

    (dst[N_COLUMNS - 2][row_index], dst[N_COLUMNS - 1][row_index])
}

pub fn write_lookup_column(
    dst: &mut [BaseField],
    input_trace: &[Vec<BaseField>],
    column_offset: usize,
    alpha: BaseField,
    z: BaseField,
) {
    let mut prev_value = BaseField::one();
    for (i, cell) in dst.iter_mut().enumerate() {
        let row_i_0 = input_trace[column_offset][i];
        let row_i_1 = input_trace[column_offset + 1][i];
        *cell = (row_i_0 + alpha * row_i_1 - z) * prev_value;
        prev_value = *cell;
    }
}