use std::cmp::Ordering;
use std::error::Error;
use find_peaks::PeakFinder;
use ndarray::{Array1, ArrayView1};
use noisy_float::prelude::Float;
use crate::analysis::filter::{highpass_filter};
use noisy_float::types::{R64, r64};

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
    pub hr_estimate: Vec<f64>,
    pub signal_quality: Vec<f64>,
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
    pub fn analyze(&self, signal: ArrayView1<f64>) -> Results {
        self.plot_signal(signal, "Raw Signal", "signal_raw.png", None);

        let mean = signal.mean().unwrap_or(0.0);
        let normalized = signal.mapv(|a| a - mean);

        self.plot_signal(normalized.view(), "Normalized Signal", "signal_normalized.png", None);

        let filtered = self.filter(normalized.view());

        self.plot_signal(filtered.view(), "Filtered Signal", "signal_filt.png", None);

        let mad = Analysis::median_absolute_deviation(filtered.view());
        println!("Median Absolute Deviation: {}", mad);
        
        let peaks = self.find_peaks(mad, filtered.view());

        self.plot_signal(filtered.view(), "Peaks", "signal_peaks.png", Some(peaks.clone()));
        
        let (hr, diff_hr) = self.analyze_bpm(peaks);

        let in_range: Vec<bool> = hr.iter().map(|&bpm| bpm >= 40.0 && bpm <= 180.0).collect();
        let changes_in_range: Vec<bool> = diff_hr.iter().map(|&change| change.abs() <= 20.0).collect();
        
        let hr_view = Array1::from(hr.clone());
        let hr_invalid_indices: Vec<usize> = in_range.iter().enumerate().filter(|(_, &valid)| valid).map(|(i, _)| i).collect();
        self.plot_signal(hr_view.view(), "BPM","signal_bpm.png", Some(hr_invalid_indices));

        let hr_diff_view = Array1::from(diff_hr.clone());
        let hr_diff_invalid_indices: Vec<usize> = changes_in_range.iter().enumerate().filter(|(_, &valid)| valid).map(|(i, _)| i).collect();
        self.plot_signal(hr_diff_view.view(), "diff BPM" , "signal_bpm_diff.png", Some(hr_diff_invalid_indices));

        let signal_quality: Vec<f64> = hr.iter().zip(diff_hr.iter()).map(|(&bpm, &diff)| {
            if bpm >= 40.0 && bpm <= 180.0 && diff.abs() <= 20.0 {
                1.0
            } else {
                0.0
            }
        }).collect();
        
        Results {
            hr_estimate: hr,
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
        
        println!("Prominence: {}, Distance: {}, Plateau: {}", multiplier * mad, distance, plateau);
        
        let slice: &[f64] = signal.as_slice().unwrap_or_else(|| {
            eprintln!("Failed to convert signal to slice");
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
                eprintln!("Error plotting {}: {}", filename, e);
            });
        } 
    }
}