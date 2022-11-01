// * builder submodule *

use crate::snn::layer::Layer;
use crate::snn::neuron::Neuron;
use crate::snn::SNN;


/** Object containing the configuration parameters describing the SNN architecture */
pub struct SnnParams<N: Neuron> {
    pub input_dimensions: usize,            /* dimension of the network input layer */
    pub neurons: Vec<Vec<N>>,               /* neurons per each layer */
    pub extra_weights: Vec<Vec<Vec<f64>>>,  /* (positive) weights between layers */
    pub intra_weights: Vec<Vec<Vec<f64>>>,  /* (negative) weights inside the same layer */
}

/** Object for the configuration and creation of the Spiking Neural Network.
    It allows to configure the network step-by-step, adding one layer at a time,
    specifying the (extra)weights the neurons and the intra weights for each layer.
    - It follows the (fluent) Builder design pattern. */
pub struct SnnBuilder<N: Neuron> {
    params: SnnParams<N>
}

impl<N: Neuron> SnnBuilder<N> {
    pub fn new() -> Self {
        Self {
            params: SnnParams {
                input_dimensions: 0,
                neurons: vec![],
                extra_weights: vec![],
                intra_weights: vec![]
            }
        }
    }

    // (the *input dimension* of the network can be automatically inferred by the compiler)
    pub fn add_layer<const INPUT_DIM: usize>(self) -> WeightsBuilder<N, INPUT_DIM, INPUT_DIM> {
        WeightsBuilder::<N, INPUT_DIM, INPUT_DIM>::new(self.params)
    }
}

// ** fluent Builder Pattern structs **

// * Weights *
/** - INPUT_DIM: is the input dimension of the new layer
    - NET_INPUT_DIM: is the input dimension of the entire neural network */
pub struct WeightsBuilder<N: Neuron, const INPUT_DIM: usize, const NET_INPUT_DIM: usize> {
    params: SnnParams<N>
}

impl<N: Neuron, const INPUT_DIM: usize, const NET_INPUT_DIM: usize> WeightsBuilder<N, INPUT_DIM, NET_INPUT_DIM> {
    pub fn new(params: SnnParams<N>) -> Self {
        Self { params }
    }

    /** It specifies the weights of the connections between the previous layer and the new one.
        Receives an array for each layer's neuron, containing all the
        ordered weights of the connections between the neuron and its siblings */
    pub fn weights<const NUM_NEURONS: usize>(mut self, weights: [[f64; INPUT_DIM]; NUM_NEURONS])
                                         -> NeuronsBuilder<N, NUM_NEURONS, NET_INPUT_DIM> {
        let mut weights_vec : Vec<Vec<f64>> = Vec::new();

        // convert the array-like parameter into a Vec
        for neuron_weights in &weights {
            weights_vec.push(Vec::from(neuron_weights.as_slice()));
        }

        // save layer weights
        self.params.extra_weights.push(weights_vec);
        NeuronsBuilder::<N, NUM_NEURONS, NET_INPUT_DIM>::new(self.params)
    }
}

// * Neurons *
pub struct NeuronsBuilder<N: Neuron, const NUM_NEURONS: usize, const NET_INPUT_DIM: usize> {
    params: SnnParams<N>
}

impl<N: Neuron, const NUM_NEURONS: usize, const NET_INPUT_DIM: usize> NeuronsBuilder<N, NUM_NEURONS, NET_INPUT_DIM> {
    pub fn new(params: SnnParams<N>) -> Self {
        Self { params }
    }

    /** Add an array of (ordered) neurons to the layer */
    pub fn neurons(mut self, neurons: [N; NUM_NEURONS]) -> IntraWeightsBuilder<N, NUM_NEURONS, NET_INPUT_DIM> {
        self.params.neurons.push(Vec::from(neurons));
        IntraWeightsBuilder::<N, NUM_NEURONS, NET_INPUT_DIM>::new(self.params)
    }
}

// * Intra Weights *
pub struct IntraWeightsBuilder<N: Neuron, const NUM_NEURONS: usize, const NET_INPUT_DIM: usize> {
    params: SnnParams<N>
}

impl<N: Neuron, const NUM_NEURONS: usize, const NET_INPUT_DIM: usize>
    IntraWeightsBuilder<N, NUM_NEURONS, NET_INPUT_DIM> {
    pub fn new(params: SnnParams<N>) -> Self {
        Self { params }
    }

    /** It specifies the (negative) weights of the connections between neurons in the same layer
        It receive a matrix-like argument, an array containing an array for each neuron where to specify
        the weights of the connections to its siblings
        Note: the array element corresponding to the link of a neuron to itself will be ignored
        (it could be set to 0). Eg: in a layer with 3 neurons, an example of intra weights matrix could be:
        [[0, -0.1, -0.3], [-0.2, 0, -0.7], [-0.9, -0.4, 0]]. The y_th element in the x_th array represent the
        weight of the link from the neuron X to the neuron Y. */

    /* *Maybe it could be better to check that for each neuron, the corresponding intra-weight is zero -> runtime error
       or maybe at runtime we cannot consider that value* */

    pub fn intra_weights(mut self, intra_weights: [[f64; NUM_NEURONS]; NUM_NEURONS])
                    -> LayerBuilder<N, NUM_NEURONS, NET_INPUT_DIM> {
        let mut intra_weights_vec : Vec<Vec<f64>> = Vec::new();

        // convert array-like intra weights parameter into a Vec
        for neuron_intra_weights in &intra_weights {
            intra_weights_vec.push(Vec::from(neuron_intra_weights.as_slice()));
        }

        // save layer intra weights
        self.params.intra_weights.push(intra_weights_vec);
        LayerBuilder::<N, NUM_NEURONS, NET_INPUT_DIM>::new(self.params)
    }
}

// * Layer *
/** It allows to add a new layer, or to build and get the SNN with the characteristics defined so far */
pub struct LayerBuilder<N: Neuron, const OUTPUT_DIM: usize, const NET_INPUT_DIM: usize> {
    params: SnnParams<N>
}

impl<N: Neuron, const OUTPUT_DIM: usize, const NET_INPUT_DIM: usize> LayerBuilder<N, OUTPUT_DIM, NET_INPUT_DIM> {
    pub fn new(params: SnnParams<N>) -> Self {
        Self { params }
    }

    /** Create and initialize the whole Spiking Neural Network with the characteristics defined so far */
    pub fn build(self) -> SNN<N, NET_INPUT_DIM, OUTPUT_DIM> {
        if  self.params.neurons.len() != self.params.extra_weights.len() &&
            self.params.neurons.len() != self.params.intra_weights.len() {
            // it must not happen
            panic!("Error: the number of neurons layers does not correspond to the number of weights layers")
        }

        let mut layers: Vec<Layer<N>> = Vec::new();

        let mut neurons_iter = self.params.neurons.into_iter();
        let mut extra_weigths_iter = self.params.extra_weights.into_iter();
        let mut intra_weights_iter = self.params.intra_weights.into_iter();

        // * retrieve the Neurons, the extra weights and the intra weights for each layer *
        while let Some(layer_neurons) = neurons_iter.next() {
            let layer_extra_weights = extra_weigths_iter.next().unwrap();
            let layer_intra_weights = intra_weights_iter.next().unwrap();

            // create and save the new layer
            let new_layer = Layer::new(layer_neurons, layer_extra_weights, layer_intra_weights);
            layers.push(new_layer);
        }

        SNN::<N, NET_INPUT_DIM, OUTPUT_DIM>::new(layers)
    }

    /** Add a new layer to the SNN */
    pub fn add_layer(self) -> WeightsBuilder<N, OUTPUT_DIM, NET_INPUT_DIM> {
        WeightsBuilder::<N, OUTPUT_DIM, NET_INPUT_DIM>::new(self.params)
    }
}