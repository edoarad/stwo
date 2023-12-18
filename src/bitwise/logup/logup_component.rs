use crate::core::air::definition::{
    Column, ColumnKind, Component, ComponentInstance, Constraint, InteractionElement,
};
use crate::core::air::graph::{GraphNode, OpParam};

pub fn create_logup_component_definition(n_bits: u32) -> Component {
    Component {
        name: "LogUp".to_string(),
        version: "0.1".to_string(),
        description: "Hand written logup component".to_string(),
        instances: vec![ComponentInstance {
            n_bits,
            generation_graph: vec![
                GraphNode {
                    name: "values".to_string(),
                    description: "Input values".to_string(),
                    size: 1 << n_bits,
                    ty: "M31".to_string(),
                    op: "generation_input".to_string(),
                    params: vec![OpParam::String("values".to_string())],
                    inputs: vec![],
                },
                GraphNode {
                    name: "zero".to_string(),
                    description: "Zero value".to_string(),
                    size: 1,
                    ty: "QM31".to_string(),
                    op: "constant".to_string(),
                    params: vec![OpParam::Int(0)],
                    inputs: vec![],
                },
                GraphNode {
                    name: "one".to_string(),
                    description: "One value".to_string(),
                    size: 1,
                    ty: "QM31".to_string(),
                    op: "constant".to_string(),
                    params: vec![OpParam::Int(1)],
                    inputs: vec![],
                },
                GraphNode {
                    name: "multiple_ones".to_string(),
                    description: "[1; n_bits]".to_string(),
                    size: 1 << n_bits,
                    ty: "QM31".to_string(),
                    op: "repeat".to_string(),
                    params: vec![
                        OpParam::String("one".to_string()),
                        OpParam::Int(1),
                        OpParam::Int(1 << n_bits),
                    ],
                    inputs: vec![],
                },
                GraphNode {
                    name: "log_up_random_shift".to_string(),
                    description: "[lambda; n_bits]".to_string(),
                    size: 1 << n_bits,
                    ty: "QM31".to_string(),
                    op: "repeat".to_string(),
                    params: vec![
                        OpParam::String("log_up_shift_element".to_string()),
                        OpParam::Int(1),
                        OpParam::Int(1 << n_bits),
                    ],
                    inputs: vec![],
                },
                GraphNode {
                    name: "shifted_values".to_string(),
                    description: "[(x - lambda); for x in values]".to_string(),
                    size: 1 << n_bits,
                    ty: "QM31".to_string(),
                    op: "sub".to_string(),
                    params: vec![],
                    inputs: vec!["values".to_string(), "log_up_random_shift".to_string()],
                },
                GraphNode {
                    name: "inverse_shifted_values".to_string(),
                    description: "[(x - lambda)^-1; for x in values]".to_string(),
                    size: 1 << n_bits,
                    ty: "QM31".to_string(),
                    op: "div".to_string(),
                    params: vec![],
                    inputs: vec!["multiple_ones".to_string(), "shifted_values".to_string()],
                },
                GraphNode {
                    name: "partial_sums".to_string(),
                    description: "The partial sums of the shifted inverses of the values"
                        .to_string(),
                    size: (1 << n_bits) + 1,
                    ty: "QM31".to_string(),
                    op: "concat".to_string(),
                    params: vec![],
                    inputs: vec!["zero".to_string(), "partial_sums_rec".to_string()],
                },
                GraphNode {
                    name: "partial_sums0".to_string(),
                    description: "The partial sums of the shifted inverses of the values"
                        .to_string(),
                    size: 1 << n_bits,
                    ty: "QM31".to_string(),
                    op: "slice".to_string(),
                    params: vec![OpParam::Int(0), OpParam::Int(1 << n_bits), OpParam::Int(1)],
                    inputs: vec!["partial_sums".to_string()],
                },
                GraphNode {
                    name: "partial_sums_rec".to_string(),
                    description: "The partial sums of the shifted inverses of the values"
                        .to_string(),
                    size: 1 << n_bits,
                    ty: "QM31".to_string(),
                    op: "add".to_string(),
                    params: vec![],
                    inputs: vec![
                        "partial_sums0".to_string(),
                        "inverse_shifted_values".to_string(),
                    ],
                },
                GraphNode {
                    name: "sum".to_string(),
                    description: "The total sums of the shifted inverses of the values".to_string(),
                    size: 1,
                    ty: "QM31".to_string(),
                    op: "slice".to_string(),
                    params: vec![
                        OpParam::Int(1 << n_bits),
                        OpParam::Int((1 << n_bits) + 1),
                        OpParam::Int(1),
                    ],
                    inputs: vec!["partial_sums_rec".to_string()],
                },
            ],
            columns: vec![
                Column {
                    name: "input values".to_string(),
                    description: "values".to_string(),
                    generation_node: "values".to_string(),
                    kind: ColumnKind::Witness,
                },
                Column {
                    name: "partial_sums_rec".to_string(),
                    description: "The partial sums of the shifted inverses of the input values"
                        .to_string(),
                    generation_node: "partial_sums_rec".to_string(),
                    kind: ColumnKind::Witness,
                },
            ],
            outputs: vec!["sum".to_string()],
            constraint_graph: vec![
                GraphNode {
                    name: "log_up_random_shift".to_string(),
                    description: "[lambda; n_bits]".to_string(),
                    size: 1 << n_bits,
                    ty: "QM31".to_string(),
                    op: "repeat".to_string(),
                    params: vec![
                        OpParam::String("log_up_shift_element".to_string()),
                        OpParam::Int(1),
                        OpParam::Int(1 << n_bits),
                    ],
                    inputs: vec![],
                },
                GraphNode {
                    name: "shifted_values".to_string(),
                    description: "[(x - lambda); for x in values]".to_string(),
                    size: 1 << n_bits,
                    ty: "QM31".to_string(),
                    op: "sub".to_string(),
                    params: vec![],
                    inputs: vec!["values".to_string(), "log_up_random_shift".to_string()],
                },
                GraphNode {
                    name: "shifted_values0".to_string(),
                    description: "[x0 - lambda]".to_string(),
                    size: 1,
                    ty: "QM31".to_string(),
                    op: "slice".to_string(),
                    params: vec![OpParam::Int(0), OpParam::Int(1), OpParam::Int(1)],
                    inputs: vec!["shifted_values".to_string()],
                },
                GraphNode {
                    name: "partial_sums".to_string(),
                    description: "C[0:N+1]".to_string(),
                    size: (1 << n_bits) + 1,
                    ty: "QM31".to_string(),
                    op: "concat".to_string(),
                    params: vec![],
                    inputs: vec!["zero".to_string(), "partial_sums_rec".to_string()],
                },
                GraphNode {
                    name: "partial_sums0".to_string(),
                    description: "C[0]".to_string(),
                    size: 1,
                    ty: "QM31".to_string(),
                    op: "slice".to_string(),
                    params: vec![OpParam::Int(0), OpParam::Int(1), OpParam::Int(1)],
                    inputs: vec!["partial_sums".to_string()],
                },
                GraphNode {
                    name: "partial_sums_slice".to_string(),
                    description: "C[0:N]".to_string(),
                    size: 1 << n_bits,
                    ty: "QM31".to_string(),
                    op: "slice".to_string(),
                    params: vec![OpParam::Int(0), OpParam::Int(1 << n_bits), OpParam::Int(1)],
                    inputs: vec!["partial_sums".to_string()],
                },
                GraphNode {
                    name: "partial_sums_consecutive_sub".to_string(),
                    description: "C_{i+1} - C_i".to_string(),
                    size: 1 << n_bits,
                    ty: "QM31".to_string(),
                    op: "sub".to_string(),
                    params: vec![],
                    inputs: vec![
                        "partial_sums_rec".to_string(),
                        "partial_sums_slice".to_string(),
                    ],
                },
                GraphNode {
                    name: "consecutive_sub_mul_shifted_values".to_string(),
                    description: "(C_{i+1} - C_i) * (x - lambda)".to_string(),
                    size: 1 << n_bits,
                    ty: "QM31".to_string(),
                    op: "mul".to_string(),
                    params: vec![],
                    inputs: vec![
                        "partial_sums_consecutive_sub".to_string(),
                        "shifted_values".to_string(),
                    ],
                },
                GraphNode {
                    name: "partial_sum_step".to_string(),
                    description: "((C_{i+1} - C_i) * (x - lambda)) - 1".to_string(),
                    size: 1 << n_bits,
                    ty: "QM31".to_string(),
                    op: "sub".to_string(),
                    params: vec![],
                    inputs: vec![
                        "multiple_ones".to_string(),
                        "consecutive_sub_mul_shifted_values".to_string(),
                    ],
                },
                GraphNode {
                    name: "first_value_check1".to_string(),
                    description: "Check that the first value is correct".to_string(),
                    size: 1,
                    ty: "QM31".to_string(),
                    op: "mul".to_string(),
                    params: vec![],
                    inputs: vec!["partial_sums0".to_string(), "shifted_values0".to_string()],
                },
                GraphNode {
                    name: "first_value_check".to_string(),
                    description: "Check that the first value is correct".to_string(),
                    size: 1,
                    ty: "QM31".to_string(),
                    op: "sub".to_string(),
                    params: vec![],
                    inputs: vec!["first_value_check1".to_string(), "one".to_string()],
                },
            ],
            constraints: vec![
                // First.
                Constraint {
                    name: "first_value_check".to_string(),
                    description: "Check that the first partial sum value is correct.".to_string(),
                    constraint_node: "first_value_check".to_string(),
                },
                // Step.
                Constraint {
                    name: "partial_sum_step".to_string(),
                    description: "Check that the partial sum values is correct.".to_string(),
                    constraint_node: "partial_sum_step".to_string(),
                },
            ],
            interaction_elements: vec![InteractionElement {
                name: "log_up_shift_element".to_string(),
                description: "Random element for shifting the logup".to_string(),
                witness_dependencies: vec!["values".to_string()],
            }],
        }],
    }
}
