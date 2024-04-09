use itertools::Itertools;
use num_traits::{One, Zero};
use tracing::{span, Level};

use super::component::{WideFibAir, WideFibComponent};
use crate::core::air::accumulation::{DomainEvaluationAccumulator, PointEvaluationAccumulator};
use crate::core::air::mask::fixed_mask_points;
use crate::core::air::{Air, Component, ComponentTrace};
use crate::core::backend::avx512::qm31::PackedSecureField;
use crate::core::backend::avx512::{AVX512Backend, BaseFieldVec, PackedBaseField, VECS_LOG_SIZE};
use crate::core::backend::{Col, Column, ColumnOps};
use crate::core::circle::CirclePoint;
use crate::core::constraints::coset_vanishing;
use crate::core::fields::m31::BaseField;
use crate::core::fields::qm31::SecureField;
use crate::core::fields::{FieldExpOps, FieldOps};
use crate::core::poly::circle::{CanonicCoset, CircleEvaluation};
use crate::core::poly::BitReversedOrder;
use crate::core::ColumnVec;
use crate::examples::wide_fibonacci::component::N_COLUMNS;

impl Air<AVX512Backend> for WideFibAir {
    fn components(&self) -> Vec<&dyn Component<AVX512Backend>> {
        vec![&self.component]
    }
}

pub fn gen_trace(
    log_size: usize,
) -> ColumnVec<CircleEvaluation<AVX512Backend, BaseField, BitReversedOrder>> {
    assert!(log_size >= VECS_LOG_SIZE);
    let mut trace = (0..N_COLUMNS)
        .map(|_| Col::<AVX512Backend, BaseField>::zeros(1 << log_size))
        .collect_vec();
    for vec_index in 0..(1 << (log_size - VECS_LOG_SIZE)) {
        let mut a = PackedBaseField::one();
        let mut b = PackedBaseField::from_array(std::array::from_fn(|i| {
            BaseField::from_u32_unchecked((vec_index * 16 + i) as u32)
        }));
        trace[0].data[vec_index] = a;
        trace[1].data[vec_index] = b;
        trace.iter_mut().skip(2).for_each(|col| {
            (a, b) = (b, a.square() + b.square());
            col.data[vec_index] = b;
        });
    }
    let domain = CanonicCoset::new(log_size as u32).circle_domain();
    trace
        .into_iter()
        .map(|eval| CircleEvaluation::<AVX512Backend, _, BitReversedOrder>::new(domain, eval))
        .collect_vec()
}

impl Component<AVX512Backend> for WideFibComponent {
    fn n_constraints(&self) -> usize {
        N_COLUMNS - 1
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_column_size() + 1
    }

    fn trace_log_degree_bounds(&self) -> Vec<u32> {
        vec![self.log_column_size(); N_COLUMNS]
    }

    fn evaluate_constraint_quotients_on_domain(
        &self,
        trace: &ComponentTrace<'_, AVX512Backend>,
        evaluation_accumulator: &mut DomainEvaluationAccumulator<AVX512Backend>,
    ) {
        assert_eq!(trace.polys.len(), N_COLUMNS);
        // TODO(spapini): Steal evaluation from commitment.
        let eval_domain = CanonicCoset::new(self.log_column_size() + 1).circle_domain();
        let trace_eval = &trace.evals;

        // Denoms.
        let span = span!(Level::INFO, "Constraint eval denominators").entered();
        // TODO(spapini): Make this prettier.
        let zero_domain = CanonicCoset::new(self.log_column_size()).coset;
        let mut denoms =
            BaseFieldVec::from_iter(eval_domain.iter().map(|p| coset_vanishing(zero_domain, p)));
        <AVX512Backend as ColumnOps<BaseField>>::bit_reverse_column(&mut denoms);
        let mut denom_inverses = BaseFieldVec::zeros(denoms.len());
        <AVX512Backend as FieldOps<BaseField>>::batch_inverse(&denoms, &mut denom_inverses);
        span.exit();

        let _span = span!(Level::INFO, "Constraint pointwise eval").entered();

        let constraint_log_degree_bound =
            Component::<AVX512Backend>::max_constraint_log_degree_bound(self);
        let n_constraints = Component::<AVX512Backend>::n_constraints(self);
        let [accum] =
            evaluation_accumulator.columns([(constraint_log_degree_bound, n_constraints)]);

        for vec_row in 0..(1 << (eval_domain.log_size() - VECS_LOG_SIZE as u32)) {
            // Numerator.
            let a = trace_eval[0].data[vec_row];
            let mut row_res =
                PackedSecureField::from_packed_m31s([
                    a - PackedBaseField::one(),
                    PackedBaseField::zero(),
                    PackedBaseField::zero(),
                    PackedBaseField::zero(),
                ]) * PackedSecureField::broadcast(accum.random_coeff_powers[N_COLUMNS - 2]);

            let mut a_sq = a.square();
            let mut b_sq = trace_eval[1].data[vec_row].square();
            #[allow(clippy::needless_range_loop)]
            for i in 0..(N_COLUMNS - 2) {
                unsafe {
                    let c = *trace_eval.get_unchecked(i + 2).data.get_unchecked(vec_row);
                    row_res +=
                        PackedSecureField::broadcast(accum.random_coeff_powers[N_COLUMNS - 3 - i])
                            * (a_sq + b_sq - c);
                    (a_sq, b_sq) = (b_sq, c.square());
                }
            }

            accum.col.set_packed(
                vec_row,
                accum.col.packed_at(vec_row) + row_res * denom_inverses.data[vec_row],
            )
        }
    }

    fn mask_points(
        &self,
        point: CirclePoint<SecureField>,
    ) -> ColumnVec<Vec<CirclePoint<SecureField>>> {
        fixed_mask_points(&vec![vec![0_usize]; N_COLUMNS], point)
    }

    fn evaluate_constraint_quotients_at_point(
        &self,
        point: CirclePoint<SecureField>,
        mask: &ColumnVec<Vec<SecureField>>,
        evaluation_accumulator: &mut PointEvaluationAccumulator,
    ) {
        let zero_domain = CanonicCoset::new(self.log_column_size()).coset;
        let denominator = coset_vanishing(zero_domain, point);
        evaluation_accumulator.accumulate((mask[0][0] - SecureField::one()) / denominator);
        for i in 0..(N_COLUMNS - 2) {
            let numerator = mask[i][0].square() + mask[i + 1][0].square() - mask[i + 2][0];
            evaluation_accumulator.accumulate(numerator / denominator);
        }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx512f"))]
#[cfg(test)]
mod tests {
    use tracing::{span, Level};

    use crate::commitment_scheme::blake2_hash::Blake2sHasher;
    use crate::commitment_scheme::hasher::Hasher;
    use crate::core::backend::avx512::AVX512Backend;
    use crate::core::channel::{Blake2sChannel, Channel};
    use crate::core::fields::m31::BaseField;
    use crate::core::fields::IntoSlice;
    use crate::core::prover::{prove, verify};
    use crate::examples::wide_fibonacci::avx::{gen_trace, WideFibAir};
    use crate::examples::wide_fibonacci::component::{WideFibComponent, LOG_N_COLUMNS};

    #[test_log::test]
    fn test_avx_wide_fib_prove() {
        // Note: To see time measurement, run test with
        //   RUST_LOG_SPAN_EVENTS=enter,close RUST_LOG=info RUST_BACKTRACE=1 RUSTFLAGS="
        //   -C target-cpu=native -C target-feature=+avx512f -C opt-level=2" cargo test
        //   test_avx_wide_fib_prove -- --nocapture

        // Note: 17 means 128MB of trace.
        const LOG_N_ROWS: u32 = 12;
        let component = WideFibComponent {
            log_fibonacci_size: LOG_N_COLUMNS as u32,
            log_n_instances: LOG_N_ROWS,
        };
        let span = span!(Level::INFO, "Trace generation").entered();
        let trace = gen_trace(component.log_column_size() as usize);
        span.exit();
        let channel = &mut Blake2sChannel::new(Blake2sHasher::hash(BaseField::into_slice(&[])));
        let air = WideFibAir { component };
        let proof = prove::<AVX512Backend>(&air, channel, trace).unwrap();

        let channel = &mut Blake2sChannel::new(Blake2sHasher::hash(BaseField::into_slice(&[])));
        verify(proof, &air, channel).unwrap();
    }
}