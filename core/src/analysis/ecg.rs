use std::cmp::Ordering;
use std::error::Error;
use std::string::ToString;
use find_peaks::PeakFinder;
use ndarray::{Array1, ArrayView1};
use noisy_float::prelude::Float;
use crate::analysis::filter::{highpass_filter};
use noisy_float::types::{R64, r64};
use slog::{Logger, o, trace, error};
use crate::log::create_logger;

#[derive(Debug, PartialEq, Clone)]
pub struct Parameters {
    pub sampling_frequency: f64,
    pub filter_cutoff_low: f64,
    pub filter_order: u32,
    pub r_peak_prominence_mad_multiple: f64,
    pub r_peak_distance: u32,
    pub r_peak_plateau: u32,
    pub hr_min: f64,
    pub hr_max: f64,
    pub hr_max_diff: f64,
}

pub struct Results {
    pub hr_estimate: f64,
    pub signal_quality: f64,
}

pub struct Analysis {
    pub(crate) params: Parameters,
    pub(crate) logger: Logger,

    pub(crate) plotter: Option<Box<dyn Fn(
        ArrayView1<f64>,
        &str,
        &str,
        Option<Vec<usize>>
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

    pub fn analyze(&self, signal: Vec<f64>) -> Results {
        let signal = Array1::from(signal);
        self.analyze_view(signal.view())
    }
    
    pub fn analyze_view(&self, signal: ArrayView1<f64>) -> Results {
        self.plot_signal(signal, "Raw Signal", "signal_raw.png", None);

        let mean = signal.mean().unwrap_or(0.0);
        let normalized = signal.mapv(|a| a - mean);

        self.plot_signal(normalized.view(), "Normalized Signal", "signal_normalized.png", None);

        let filtered = self.filter(normalized.view());

        self.plot_signal(filtered.view(), "Filtered Signal", "signal_filt.png", None);

        let mad = Analysis::median_absolute_deviation(filtered.view());
        trace!(self.logger, "Median Absolute Deviation"; "mad" => mad);

        let peaks = self.find_peaks(mad, filtered.view());

        self.plot_signal(filtered.view(), "Peaks", "signal_peaks.png", Some(peaks.clone()));
        
        let (hr, diff_hr) = self.analyze_bpm(peaks);

        let hr_min = self.params.hr_min;
        let hr_max = self.params.hr_max;
        let hr_max_diff = self.params.hr_max_diff;
        let in_range: Vec<bool> = hr.iter().map(|&bpm| bpm >= hr_min && bpm <= hr_max).collect();
        let changes_in_range: Vec<bool> = diff_hr.iter().map(|&change| change.abs() <= hr_max_diff).collect();
        
        // only for plotting
        let hr_view = Array1::from(hr.clone());
        let hr_valid_indices: Vec<usize> = in_range.iter().enumerate().filter(|(_, &valid)| valid).map(|(i, _)| i).collect();
        self.plot_signal(hr_view.view(), "BPM","signal_bpm.png", Some(hr_valid_indices));
        let hr_diff_view = Array1::from(diff_hr.clone());
        let hr_diff_valid_indices: Vec<usize> = changes_in_range.iter().enumerate().filter(|(_, &valid)| valid).map(|(i, _)| i).collect();
        self.plot_signal(hr_diff_view.view(), "diff BPM" , "signal_bpm_diff.png", Some(hr_diff_valid_indices));

        let valid_pulse_count = in_range.iter().zip(changes_in_range.iter()).map(|(&a, &b)| a && b).count();

        // ensure we have a lower bound on the number of pulses
        let signal_duration = signal.len() as f64 / self.params.sampling_frequency;
        let pulse_width_max = 60.0 / self.params.hr_min;
        let min_pulses = signal_duration / pulse_width_max;

        let hr_estimate = hr.iter().sum::<f64>() / hr.len() as f64;
        let signal_quality = valid_pulse_count as f64 / f64::max(min_pulses, hr.len() as f64);
        
        Results {
            hr_estimate,
            signal_quality,
        }
    }

    fn filter(&self, signal: ArrayView1<f64>) -> Array1<f64> {
        let low = self.params.filter_cutoff_low;
        let order = self.params.filter_order as usize;
        let fs = self.params.sampling_frequency;
        highpass_filter(signal, low, order, fs)
    }

    fn median_absolute_deviation(data: ArrayView1<f64>) -> f64 {
        let n = data.len();
        if n == 0 {
            return f64::NAN;
        }

        // Convert f64 values to orderable R64 values
        let mut noisy_data: Vec<R64> = data.iter().map(|&x| r64(x)).collect();

        // Sort the noisy data to find the median
        noisy_data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        // Calculate median
        let median = if n % 2 == 0 {
            (noisy_data[n / 2 - 1] + noisy_data[n / 2]) / 2.0
        } else {
            noisy_data[n / 2]
        };

        // Calculate the absolute deviations from the median
        let deviations: Vec<f64> = noisy_data
            .iter()
            .map(|&x| (x - median).abs().raw())
            .collect();

        // Convert deviations to orderable R64 values for sorting
        let mut noisy_deviations: Vec<R64> = deviations.iter().map(|&x| r64(x)).collect();

        // Sort deviations to find the median absolute deviation
        noisy_deviations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        // Calculate the median of the absolute deviations
        if n % 2 == 0 {
            ((noisy_deviations[n / 2 - 1] + noisy_deviations[n / 2]) / 2.0).raw()
        } else {
            noisy_deviations[n / 2].raw()
        }
    }
    
    fn find_peaks(&self, mad: f64, signal: ArrayView1<f64>) -> Vec<usize> {
        let multiplier = self.params.r_peak_prominence_mad_multiple;
        let distance = self.params.r_peak_distance as usize;
        let plateau = self.params.r_peak_plateau as usize;

        trace!(self.logger, "Peak Finding Parameters"; "Prominence" => multiplier * mad, "Distance" => distance, "Plateau" => plateau);

        let slice: &[f64] = signal.as_slice().unwrap_or_else(|| {
            error!(self.logger, "Failed to convert signal to slice");
            &[]
        });
        let peaks = PeakFinder::new(slice)
            .with_min_prominence(multiplier * mad)
            .with_min_distance(distance)
            .with_max_plateau_size(plateau)
            .find_peaks();
        
        let mut peaks: Vec<usize> = peaks.iter().map(|p| p.position.start).collect();
        peaks.sort_unstable();
        peaks
    }

    fn analyze_bpm(&self, peak_indices: Vec<usize>) -> (Vec<f64>, Vec<f64>) {
        let fs = self.params.sampling_frequency;
        
        let peak_distances = peak_indices.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();
        let peak_bpm: Vec<f64> = peak_distances.iter().map(|&d| 60.0 * fs / d as f64).collect();

        let mut bpm_changes: Vec<f64> = peak_bpm.windows(2).map(|w| w[1] - w[0]).collect();
        bpm_changes.insert(0, 0.0); // To keep the same length as peak_bpm
        
        (peak_bpm, bpm_changes)
    }

    fn plot_signal(&self, signal: ArrayView1<f64>, title: &str, filename: &str, points: Option<Vec<usize>>)  {
        if let Some(f) = &self.plotter {
            f(signal, title, filename, points).unwrap_or_else(|e| {
                error!(self.logger, "Error plotting {}: {}", filename, e);
            });
        } 
    }
}