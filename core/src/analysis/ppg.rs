use std::cmp::Ordering;
use std::error::Error;
use ndarray::{Array1, ArrayView1, s};
use slog::{error, Logger, o, trace};
use crate::analysis::filter::{bandpass_filter, lower_envelope_est};
use crate::log::create_logger;

#[derive(Debug, PartialEq, Clone)]
pub struct Parameters {
    pub sampling_frequency: f64,
    pub filter_cutoff_low: f64,
    pub filter_cutoff_high: f64,
    pub filter_order: u32,
    pub envelope_range: u16,
    pub amplitude_min: f64,
    pub amplitude_max: f64,
    pub trough_depth_min: f64,
    pub trough_depth_max: f64,
    pub pulse_width_min: f64,
    pub pulse_width_max: f64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Results {
    pub hr_estimate: f64,
    pub signal_quality: f64,
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
    pub(crate) params: Parameters,
    pub(crate) logger: Logger,

    pub(crate) plotter: Option<Box<dyn Fn(
        ArrayView1<f64>,
        &str,
        &str,
        Option<Vec<usize>>,
    ) -> Result<(), Box<dyn Error>> + Send + Sync>>,
}

impl Analysis {
    pub fn new_with_logs(params: Parameters, logger: Logger) -> Self {
        Self {
            params,
            logger,
            plotter: None,
        }
    }

    pub fn new(params: Parameters) -> Self {
        let logger = create_logger("analysis".to_string())
            .new(o!("module" => "ppg-analysis"));
        Self::new_with_logs(params, logger)
    }

    pub fn analyze(&self, signal: Vec<f64>) -> Option<Results> {
        let signal = Array1::from(signal);
        self.analyze_view(signal.view())
    }

    pub fn analyze_view(&self, signal: ArrayView1<f64>) -> Option<Results> {
        self.plot_signal(signal, "Raw Signal", "signal_raw.png", None);

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
            return None;
        }

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

        trace!(self.logger, "Test Results";
            "amplitudes" => format!("{:?}", pulses.iter().map(|p| p.amplitude()).collect::<Vec<f64>>()),
            "trough_depth_differences" => format!("{:?}", pulses.iter().map(|p| p.trough_depth_difference()).collect::<Vec<f64>>()), 
            "relative_depth_differences" => format!("{:?}", pulses.iter().map(|p| p.relative_depth_difference()).collect::<Vec<f64>>()),
            "pulse_widths" => format!("{:?}", pulses.iter().map(|p| p.pulse_width(self.params.sampling_frequency as f64)).collect::<Vec<f64>>()), 
            "valid_pulses" => format!("{:?}", valid_by_thresholds)
        );
        
        let hr_estimate = 60.0 / pulses.iter().map(|p| p.pulse_width(self.params.sampling_frequency)).sum::<f64>() / (pulses.len() as f64);
        let valid_pulse_count = valid_by_thresholds.iter().filter(|&&(v1, v2, v3)| v1 && v2 && v3).count();
        // by sampling freq, max pulse width, signal length
        
        // ensure we have a lower bound on the number of pulses
        let signal_duration = signal.len() as f64 / self.params.sampling_frequency;
        let min_pulses = signal_duration / self.params.pulse_width_max;
        
        let signal_quality = valid_pulse_count as f64 / f64::max(min_pulses, pulses.len() as f64);
        
        
        Some(Results {
            hr_estimate,
            signal_quality,
        })

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
        let amplitude_min = self.params.amplitude_min;
        let amplitude_max = self.params.amplitude_max;
        let trough_depth_min = self.params.trough_depth_min;
        let trough_depth_max = self.params.trough_depth_max;
        let pulse_width_min = self.params.pulse_width_min;
        let pulse_width_max = self.params.pulse_width_max;
        
        pulses.iter().map(|pulse| {
            let amplitude_valid = amplitude_min <= pulse.amplitude() && pulse.amplitude() <= amplitude_max;
            let trough_depth_valid = (trough_depth_min..=trough_depth_max).contains(&pulse.relative_depth_difference());
            let pulse_width_valid = (pulse_width_min..=pulse_width_max).contains(&pulse.pulse_width(self.params.sampling_frequency));

            (amplitude_valid, trough_depth_valid, pulse_width_valid)
        }).collect()
    }

    fn plot_signal(&self, signal: ArrayView1<f64>, title: &str, filename: &str, points: Option<Vec<usize>>) {
        if let Some(f) = &self.plotter {
            f(signal, title, filename, points).unwrap_or_else(|e| {
                error!(self.logger, "Error plotting {}", filename; "error" => e.to_string());
            });
        }
    }
}
