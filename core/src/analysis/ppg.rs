use ndarray::{Array1, ArrayView1, s, Zip};
use crate::analysis::filter::{bandpass_filter, lower_envelope_est};
use crate::analysis::plotters::plot_signal_f64;

// Assume additional necessary imports for DSP functionality

pub struct Parameters {
    pub sampling_frequency: f64,
    pub filter_cutoff_low: f64,
    pub filter_cutoff_high: f64,
    pub filter_order: usize,
    pub envelope_range: u16,
    pub amplitude_min: i32,
    pub amplitude_max: i32,
}

pub struct Results {
    pub hr_estimate: Vec<u16>,
    pub signal_quality: Vec<u16>,
}

pub struct Pulse<'a> {
    signal: ArrayView1<'a, f64>,
    start_trough_index: usize,
    peak_index: usize,
    end_trough_index: usize,
}

impl<'a> Pulse<'a> {
    pub fn new(signal: ArrayView1<'a, f64>, start: usize, peak: usize, end: usize) -> Self {
        Pulse {
            signal: signal,
            start_trough_index: start,
            peak_index: peak,
            end_trough_index: end,
        }
    }
    
    // Methods to access pulse characteristics
    pub fn amplitude(&self) -> f64 {
        self.signal[self.peak_index] - self.signal[self.start_trough_index]
    }

    pub fn trough_depth_difference(&self) -> f64 {
        self.signal[self.end_trough_index] - self.signal[self.start_trough_index]
    }

    pub fn relative_depth_difference(&self) -> f64 {
        let depth = self.trough_depth_difference();
        let amplitude = self.amplitude();
        depth / amplitude
    }

    pub fn pulse_width(&self, fs: f64) -> f64 {
        (self.end_trough_index - self.start_trough_index) as f64 / fs
    }
}

pub(crate) struct Analysis {
    pub(crate) params: Parameters,
}

impl Analysis {
    pub fn analyze(&self, signal: Array1<u16>) -> Results {
        let signal = signal.map(|&x| -(x as f64));
        let stripped = signal.view();
        //plot_signal_f64(stripped, "signal_raw.png", None);
        
        let mean = stripped.mean().unwrap();
        let normalized = stripped.mapv(|a| a - mean);
        
        //plot_signal_f64(normalized.view(), "signal_normalized.png", None);
        
        let filtered = self.filter(normalized.view());
        //plot_signal_f64(filtered.view(), "signal_filt.png", None);

        let lower_env = self.lower_envelope(&filtered);
        let pulses = self.find_pulses(filtered.view(), &lower_env);
        
        let points_trough = pulses.iter().map(|p| p.start_trough_index).collect::<Vec<_>>();
        //plot_signal_f64(filtered.view(), "signal_pulses.png", Some(points_trough));
        
        if pulses.len() < 3 {
            return Results {
                hr_estimate: vec![],
                signal_quality: vec![0; pulses.len()],
            };
        }
        
        // drop first and last pulse
        //let pulses = &pulses[1..pulses.len()-1];
        let valid_by_thresholds = self.validate_pulses(&pulses);
        
        let points_peak = pulses.iter().map(|p| p.peak_index).collect::<Vec<_>>();
        let valid_peaks = points_peak.iter().enumerate().filter_map(|(i, &p)| {
            if valid_by_thresholds[i].0 && valid_by_thresholds[i].1 && valid_by_thresholds[i].2 {
                Some(p)
            } else {
                None
            }
        }).collect::<Vec<_>>();

        //plot_signal_f64(filtered.view(), "signal_valid.png", Some(valid_peaks));

        
        println!("------------------------------------");

        println!("Amplitudes: {:?}", pulses.iter().map(|p| p.amplitude()).collect::<Vec<f64>>());
        println!("Trough depth differences: {:?}", pulses.iter().map(|p| p.trough_depth_difference()).collect::<Vec<f64>>());
        println!("Relative depth differences: {:?}", pulses.iter().map(|p| p.relative_depth_difference()).collect::<Vec<f64>>());
        println!("Pulse widths: {:?}", pulses.iter().map(|p| p.pulse_width(self.params.sampling_frequency as f64)).collect::<Vec<f64>>());
        

        println!("Valid pulses: {:?}", valid_by_thresholds);
        
        println!("------------------------------------");
        
        Results {
            hr_estimate: vec![],
            signal_quality: valid_by_thresholds.iter().map(|&(v1, v2, v3)| {
                if v1 && v2 && v3 {
                    1
                } else {
                    if !v1 {
                        2
                    } else if !v2 {
                        3
                    } else {
                        4
                    }
                }
            }).collect(),
        }
    }

    fn filter(&self, data: ArrayView1<f64>) -> Array1<f64> {
        bandpass_filter(data, self.params.filter_cutoff_low, self.params.filter_cutoff_high, self.params.sampling_frequency)
    }

    fn lower_envelope(&self, data: &Array1<f64>) -> Array1<f64> {
        lower_envelope_est(data, self.params.envelope_range as usize)
    }

    fn find_pulses<'a>(&self, signal: ArrayView1<'a, f64>, envelope: &Array1<f64>) -> Vec<Pulse<'a>> {
        let trough_indices: Vec<usize> = signal.iter().zip(envelope.iter())
            .enumerate()
            .filter_map(|(i, (&s, &e))| if s == e { Some(i) } else { None })
            .collect();

         trough_indices.windows(2)
            .map(|w| {
                let start_trough = w[0];
                let end_trough = w[1];
                
                let pulse = signal.slice(s![start_trough..=end_trough]);
                let index_peak = pulse.iter().enumerate().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).unwrap().0;
                
                Pulse::new(signal, start_trough, start_trough + index_peak, end_trough)
            }).collect()
    }

    
    fn validate_pulses(&self, pulses: &[Pulse]) -> (Vec<(bool, bool, bool)>) {
        let amplitude_min = self.params.amplitude_min as f64;
        let amplitude_max = self.params.amplitude_max as f64;
        
        pulses.iter().map(|pulse| {
            let amplitude_valid = amplitude_min <= pulse.amplitude() && pulse.amplitude() <= amplitude_max;
            let trough_depth_valid = (-0.25..=0.25).contains(&pulse.relative_depth_difference());
            let pulse_width_valid = (1.0 / 3.0..=1.5).contains(&pulse.pulse_width(self.params.sampling_frequency as f64));

            (amplitude_valid, trough_depth_valid, pulse_width_valid)
        }).collect()
    }
}
