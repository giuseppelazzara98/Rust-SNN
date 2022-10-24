
use crate::snn::layer::Layer;
use crate::snn::neuron::Neuron;

pub mod builders;
    mod layer; // private
pub mod neuron;

// * SNN module *

/**
    Object representing the Spiking Neural Network itself
    - N: is the type representing the Neuron
    - NET_INPUT_DIM: is the input dimension of the network, i.e. the size of the input layer
    - NET_OUTPUT_DIM: is the output dimension of the network, i.e. the size of the output layer
    Having a generic cons type such as NET_INPUT_DIM allows to check at compile time
    the size of the input provided by the user
*/
pub struct SNN<N: Neuron, const NET_INPUT_DIM: usize, const NET_OUTPUT_DIM: usize> {
    _layers: Vec<Layer<N>>
    // TODO Note: I removed tx and rc because it is better to create them on the fly before processing
    //            the input, as well as all the others tx(s) and rc(s) for the layers. In this way, they will be
    //            dropped as soon as they are not needed anymore (after processing the input), instead of keeping
    //            them as fixed struct fields even when the computation is done
    //            Furthermore, it simplifies the layer.process() method, because in this way the tx(s) are dropped
    //            as soon as the input is processed, and this causes the correspondent rc(s) to stop waiting in
    //            the next layer, leading that layer's thread to return - Mario
}

impl<N: Neuron, const NET_INPUT_DIM: usize, const NET_OUTPUT_DIM: usize> SNN<N, NET_INPUT_DIM, NET_OUTPUT_DIM> {
    // test
    pub fn new(_layers: Vec<Layer<N>>) -> Self {
        Self { _layers }
    }

    // spikes contains an array for each input layer's neuron, and each array has the same
    // number of spikes, equal to the duration of the input
    // (spikes is a matrix, one row for each input neuron, and one column for each time instant)
    // * this method is able to check user input at compile-time *
    // TODO Note: I don't know if these are the best input and output for this method, let's think about that - Mario
    pub fn process<const SPIKES_DURATION: usize> (&self, _spikes: &[[u8; SPIKES_DURATION]; NET_INPUT_DIM])
        -> &[[u8; SPIKES_DURATION]; NET_OUTPUT_DIM] {
        todo!()

        // TODO: encode spikes into SpikeEvent(s)

        // call: process_events(spikes)

        // TODO: decode output into array shape
    }

    pub fn process_events(&self, _spikes: Vec<SpikeEvent>) -> Vec<SpikeEvent> {
        todo!()
        // TODO: create input TX and output RC for each layers

        // TODO: spawn threads

        // TODO: fire input SpikeEvents into *net_input* tx

        // TODO: get output SpikeEvents from *net_output* rc
    }

    // (same as process(), but it checks input spikes sizes at *run-time*:
    // spikes must have a number of Vec(s) equal to NET_INPUT_DIM, and all
    // these Vec(s) must have the same length), otherwise panic!()
    pub fn process_dyn(&self, _spikes: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        todo!()
    }
}

/* Object representing the output spikes generated by a single layer */
pub struct SpikeEvent {
    _ts: u64,
    _spikes: Vec<u8>,
}

impl SpikeEvent {
    pub fn new(_ts: u64, _spikes: Vec<u8>) -> Self {
        Self { _ts, _spikes }
    }
}
