use std::sync::mpsc::channel;
use std::thread;
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
#[derive(Debug)]
pub struct SNN<N: Neuron + Clone + Send + 'static, const NET_INPUT_DIM: usize, const NET_OUTPUT_DIM: usize> {
    layers: Vec<Layer<N>>
}

impl<N: Neuron + Clone + Send + 'static, const NET_INPUT_DIM: usize, const NET_OUTPUT_DIM: usize>
    SNN<N, NET_INPUT_DIM, NET_OUTPUT_DIM> {
    // test
    pub fn new(layers: Vec<Layer<N>>) -> Self {
        Self { layers }
    }

    // spikes contains an array for each input layer's neuron, and each array has the same
    // number of spikes, equal to the duration of the input
    // (spikes is a matrix, one row for each input neuron, and one column for each time instant)
    // * this method is able to check user input at compile-time *
    pub fn process<const SPIKES_DURATION: usize> (&mut self, spikes: &[[u8; SPIKES_DURATION]; NET_INPUT_DIM])
        -> [[u8; SPIKES_DURATION]; NET_OUTPUT_DIM] {
        // * encode spikes into SpikeEvent(s) *
        let input_spike_events = SNN::<N, NET_INPUT_DIM, NET_OUTPUT_DIM>::encode_spikes(spikes);

        // * process input *
        let output_spike_events = self.process_events(input_spike_events);

        // * decode output into array shape *
        let decoded_output: [[u8; SPIKES_DURATION]; NET_OUTPUT_DIM] =
            SNN::<N, NET_INPUT_DIM, NET_OUTPUT_DIM>::decode_spikes(output_spike_events);

        decoded_output
    }

    fn process_events(&mut self, spikes: Vec<SpikeEvent>) -> Vec<SpikeEvent> {
        // create the threads' pool
        let mut threads = Vec::new();

        // create input TX and output RC for each layers and spawn layers threads
        let (net_input_tx, mut layer_rc) = channel::<SpikeEvent>();

        for layer in &mut self.layers {
            let (layer_tx, next_layer_rc) = channel::<SpikeEvent>();

            let static_layer = Box::leak(Box::new(layer.clone()));

            let thread = thread::spawn(move || {
                static_layer.process(layer_rc, layer_tx);
            });

            threads.push(thread);   // push the new thread into threads' pool
            layer_rc = next_layer_rc;    // update external rc, to pass it to the next layer
        }

        let net_output_rc = layer_rc;

        // * fire input SpikeEvents into *net_input* tx *
        for spike_event in spikes {
            let instant = spike_event.ts;
            net_input_tx.send(spike_event)
                        .expect(&format!("Unexpected error sending input spike event t={}", instant));
        }
        drop(net_input_tx); // * drop input tx, to make all the threads terminate *

        // * get output SpikeEvents from *net_output* rc *
        let mut output_events = Vec::<SpikeEvent>::new();

        while let Ok(spike_event) = net_output_rc.recv() {
            output_events.push(spike_event);
        }

        // waiting for threads to terminate
        for thread in threads {
            thread.join().unwrap();
        }

        output_events
    }

    // (same as process(), but it checks input spikes sizes at *run-time*:
    // spikes must have a number of Vec(s) equal to NET_INPUT_DIM, and all
    // these Vec(s) must have the same length), otherwise panic!()
    fn _process_dyn(&mut self, spikes: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        // check num of spikes vec(s)
        if spikes.len() != NET_INPUT_DIM {
            panic!("Error: dimensions mismatch - each input layer's neuron must have its own spikes vec");
        }

        // * encode input spikes in spike events *

        let mut spikes_events = Vec::<SpikeEvent>::new();
        let mut spikes_duration: Option<usize> = None;

        for (n, neuron_spikes) in spikes.into_iter().enumerate() {
            let temp_len = neuron_spikes.len();

            // check spikes durations - they must have all the same size
            match spikes_duration {
                None => spikes_duration = Some(temp_len),
                Some(duration) => if temp_len != duration {
                    panic!("Error: different size spikes vec(s) found \
                            - spikes must have the same duration for each input layer's neuron")
                }
            }

            if spikes_events.len() == 0 {    // the first cycle...
                // ...create the spike events
                (0..spikes_duration.unwrap()).for_each(|t| {
                    let spike_event = SpikeEvent::new(t as u64, Vec::<u8>::new());
                    spikes_events.push(spike_event);
                });
            }

            // copy each spike in the spike_events vec
            for t in 0..spikes_duration.unwrap() {
                let temp_spike = neuron_spikes[t];
                if temp_spike != 0 && temp_spike != 1 {
                    panic!("Error: input spike must be 0 or 1 for neuron {} in t={}", n, t);
                }

                spikes_events[t].spikes.push(temp_spike);
            }
        }

        // * run SNN *
        let output_spike_events = self.process_events(spikes_events);

        // * decode output spikes in spike events *

        // create and initialize output object
        let mut output_spikes: Vec<Vec<u8>> = Vec::new();

        for _ in &output_spike_events.get(0)
                                     .unwrap_or(&SpikeEvent::new(0, Vec::<u8>::new()))
                                     .spikes {
            // create as many internal Vec<u8> as the length of the first output spike_event (num of output neurons)
            output_spikes.push(Vec::<u8>::new());
        }

        // copy processed spikes in the output spikes vec
        for spike_event in output_spike_events {
            for (out_neuron_index, spike) in spike_event.spikes.into_iter().enumerate() {
                output_spikes[out_neuron_index].push(spike);
            }
        }

        output_spikes
    }

    // * private functions *
    fn encode_spikes<const SPIKES_DURATION: usize>(spikes: &[[u8; SPIKES_DURATION]; NET_INPUT_DIM])
        -> Vec<SpikeEvent> {
        let mut spike_events = Vec::<SpikeEvent>::new();

        for t in 0..SPIKES_DURATION {
            let mut t_spikes = Vec::<u8>::new();

            // retrieve the input spikes for each neuron
            for in_neuron_index in 0..NET_INPUT_DIM {
                if spikes[in_neuron_index][t] != 0 && spikes[in_neuron_index][t] != 1 {
                    panic!("Error: input spike must be 0 or 1 in position ({}, {})", in_neuron_index, t);
                }
                t_spikes.push(spikes[in_neuron_index][t]);
            }

            let t_spike_event = SpikeEvent::new(t as u64, t_spikes);
            spike_events.push(t_spike_event);
        }

        spike_events
    }

    fn decode_spikes<const SPIKES_DURATION: usize>(spikes: Vec<SpikeEvent>)
        -> [[u8; SPIKES_DURATION]; NET_OUTPUT_DIM] {
        let mut result = [[0u8; SPIKES_DURATION]; NET_OUTPUT_DIM];
        let mut out_neuron_index;

        for spike_event in spikes {
            out_neuron_index = 0;

            for spike in spike_event.spikes {
                result[out_neuron_index][spike_event.ts as usize] = spike;
                out_neuron_index += 1;
            }
        }

        result
    }
}

/* Object representing the output spikes generated by a single layer */
#[derive(Debug)]
pub struct SpikeEvent {
    ts: u64,
    spikes: Vec<u8>,
}

impl SpikeEvent {
    pub fn new(ts: u64, spikes: Vec<u8>) -> Self {
        Self { ts, spikes }
    }
}
