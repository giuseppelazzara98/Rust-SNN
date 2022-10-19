// * Neuron submodule *

/**
    Trait for the implementation of all the Neuron models.
    It represents a general Neuron of a Layer
*/
pub trait Neuron {
    fn compute_v_mem(t: u64, weighted_sum: f64) -> f64;
}
