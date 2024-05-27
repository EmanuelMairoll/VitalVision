use std::cmp::Ordering;
use std::error::Error;
use ndarray::{Array1, ArrayView1, s};
use crate::analysis::filter::{bandpass_filter, lower_envelope_est};

#[derive(Debug, PartialEq, Clone)]
pub struct Parameters {
    pub sampling_frequency: f64,
    pub filter_cutoff_low: f64,
    pub filter_cutoff_high: f64,
    pub filter_order: u32,
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
            signal,
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

pub struct Analysis {
    pub params: Parameters,

    pub plotter: Option<Box<dyn Fn(
        ArrayView1<f64>,
        &str,
        &str,
        Option<Vec<usize>>
    ) -> Result<(), Box<dyn Error>> + Send + Sync>>,
}

impl Analysis {
    pub fn analyze(&self, signal: Array1<f64>) -> Results {
        self.plot_signal(signal.view(), "Raw Signal", "signal_raw.png", None);
        
        let mean = signal.mean().unwrap();
        let normalized = signal.mapv(|a| a - mean);
        
        self.plot_signal(normalized.view(), "Normalized Signal", "signal_normalized.png", None);
        
        let filtered = self.filter(normalized.view());
        
        self.plot_signal(filtered.view(), "Filtered Signal", "signal_filt.png", None);
        
        let lower_env = self.lower_envelope(&filtered);
        let pulses = self.find_pulses(filtered.view(), &lower_env);
        
        let points_trough = pulses.iter().map(|p| p.start_trough_index).collect::<Vec<_>>();
        self.plot_signal(filtered.view(), "Pulses", "signal_pulses.png", Some(points_trough));
        
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

        self.plot_signal(filtered.view(), "Peaks of valid Pulses", "signal_valid.png", Some(valid_peaks));

        
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
        let (low, high) = (self.params.filter_cutoff_low, self.params.filter_cutoff_high);
        let order = self.params.filter_order as usize;
        let fs = self.params.sampling_frequency;
        bandpass_filter(data, low, high, order, fs)
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
                let index_peak = pulse.iter().enumerate().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal)).unwrap().0;
                
                Pulse::new(signal, start_trough, start_trough + index_peak, end_trough)
            }).collect()
    }

    
    fn validate_pulses(&self, pulses: &[Pulse]) -> Vec<(bool, bool, bool)> {
        let amplitude_min = self.params.amplitude_min as f64;
        let amplitude_max = self.params.amplitude_max as f64;
        
        pulses.iter().map(|pulse| {
            let amplitude_valid = amplitude_min <= pulse.amplitude() && pulse.amplitude() <= amplitude_max;
            let trough_depth_valid = (-0.25..=0.25).contains(&pulse.relative_depth_difference());
            let pulse_width_valid = (1.0 / 3.0..=1.5).contains(&pulse.pulse_width(self.params.sampling_frequency as f64));

            (amplitude_valid, trough_depth_valid, pulse_width_valid)
        }).collect()
    }

    fn plot_signal(&self, signal: ArrayView1<f64>, title: &str, filename: &str, points: Option<Vec<usize>>)  {
        if let Some(f) = &self.plotter {
            f(signal, title, filename, points).unwrap_or_else(|e| {
                eprintln!("Error plotting {}: {}", filename, e);
            });
        }
    }
}
