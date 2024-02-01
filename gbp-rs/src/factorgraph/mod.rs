pub mod factor;
pub mod factorgraph;
pub mod message;
pub mod variable;

type NodeId = usize;

use nutype::nutype;

/// Represents a closed interval [0,1]
#[nutype(
    validate(greater_or_equal = 0.0, less_or_equal = 1.0),
    derive(Debug, Clone, Copy)
)]
pub struct UnitInterval(f64);
