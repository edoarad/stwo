use self::column::{BaseFieldVec, SecureFieldVec};
use self::m31::PackedBaseField;
use self::qm31::PackedSecureField;
use super::ColumnOps;
use crate::core::fields::m31::BaseField;
use crate::core::fields::qm31::SecureField;
use crate::core::fields::{FieldExpOps, FieldOps};

pub mod cm31;
pub mod column;
mod lookups;
pub mod m31;
pub mod qm31;

#[derive(Copy, Clone, Debug)]
pub struct SimdBackend;

impl ColumnOps<BaseField> for SimdBackend {
    type Column = BaseFieldVec;

    fn bit_reverse_column(_column: &mut Self::Column) {
        // // Fallback to cpu bit_reverse.
        // if column.data.len().ilog2() < bit_reverse::MIN_LOG_SIZE {
        //     utils::bit_reverse(column.as_mut_slice());
        //     return;
        // }
        // bit_reverse_m31(&mut column.data);
        todo!()
    }
}

impl FieldOps<BaseField> for SimdBackend {
    fn batch_inverse(column: &Self::Column, dst: &mut Self::Column) {
        PackedBaseField::batch_inverse(&column.data, &mut dst.data);
    }
}

impl ColumnOps<SecureField> for SimdBackend {
    type Column = SecureFieldVec;

    fn bit_reverse_column(_column: &mut Self::Column) {
        // // Fallback to cpu bit_reverse.
        // // TODO(AlonH): Implement AVX512 bit_reverse for SecureField.
        // utils::bit_reverse(column.to_vec().as_mut_slice());
        todo!()
    }
}

impl FieldOps<SecureField> for SimdBackend {
    fn batch_inverse(column: &Self::Column, dst: &mut Self::Column) {
        PackedSecureField::batch_inverse(&column.data, &mut dst.data);
    }
}