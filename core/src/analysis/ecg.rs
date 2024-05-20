use std::error::Error;
use find_peaks::PeakFinder;
use ndarray::{Array1, ArrayView1};
use crate::analysis::filter::{bandpass_filter, highpass_filter};

pub struct Parameters {
    pub sampling_frequency: f64,
    pub filter_bandpass_frequencies: (f64, f64),
    pub filter_order: usize,
    pub r_peak_prominence: f64,
    pub r_peak_height: f64,
    pub r_peak_distance: usize,
    pub r_peak_width: usize,
    pub hr_range: (f64, f64),
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
        self.plot_signal(signal, "Raw Signal", "signal_raw.png", None).unwrap();

        let mean = signal.mean().unwrap();
        let normalized = signal.mapv(|a| a - mean);

        self.plot_signal(normalized.view(), "Normalized Signal", "signal_normalized.png", None).unwrap();

        let filtered = self.filter(normalized.view());

        self.plot_signal(filtered.view(), "Filtered Signal", "signal_filt.png", None).unwrap();

        let peaks = self.find_peaks(filtered.view());

        self.plot_signal(filtered.view(), "Peaks", "signal_peaks.png", Some(peaks.clone())).unwrap();
        
        let (hr, diff_hr) = self.analyze_bpm(peaks);

        let in_range: Vec<bool> = hr.iter().map(|&bpm| bpm >= 40.0 && bpm <= 180.0).collect();
        let changes_in_range: Vec<bool> = diff_hr.iter().map(|&change| change.abs() <= 20.0).collect();
        
        let hr_view = Array1::from(hr.clone());
        let hr_invalid_indices: Vec<usize> = in_range.iter().enumerate().filter(|(_, &valid)| valid).map(|(i, _)| i).collect();
        self.plot_signal(hr_view.view(), "BPM","signal_bpm.png", Some(hr_invalid_indices)).unwrap();

        let hr_diff_view = Array1::from(diff_hr.clone());
        let hr_diff_invalid_indices: Vec<usize> = changes_in_range.iter().enumerate().filter(|(_, &valid)| valid).map(|(i, _)| i).collect();
        self.plot_signal(hr_diff_view.view(), "diff BPM" , "signal_bpm_diff.png", Some(hr_diff_invalid_indices)).unwrap();

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
        let (low, high) = self.params.filter_bandpass_frequencies;
        let order = self.params.filter_order;
        let fs = self.params.sampling_frequency;
        highpass_filter(signal, low, order, fs)
    }

    fn find_peaks(&self, signal: ArrayView1<f64>) -> Vec<usize> {
        let prominence = self.params.r_peak_prominence;
        let height = self.params.r_peak_height;
        let distance = self.params.r_peak_distance;
        let width = self.params.r_peak_width;

        let slice: &[f64] = signal.as_slice().unwrap();
        let peaks = PeakFinder::new(slice)
            .with_min_prominence(prominence)
            .with_min_height(height)
            .with_min_distance(distance)
            //.with_min_plateau_size(width)
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

    fn plot_signal(&self, signal: ArrayView1<f64>, title: &str, filename: &str, points: Option<Vec<usize>>) -> Result<(), Box<dyn Error>> {
        if let Some(f) = &self.plotter {
            f(signal, title, filename, points)
        } else {
            Ok(())
        }
    }
}