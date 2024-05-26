use biquad::{Biquad, Coefficients, DirectForm1, Q_BUTTERWORTH_F64, ToHertz, Type};
use ndarray::{Array1, ArrayView1, s};
use ndarray_stats::QuantileExt;

pub fn bandpass_filter(data: ArrayView1<f64>, lowcut: f64, highcut: f64, order: usize, fs: f64) -> Array1<f64> {

    let low_coeff = Coefficients::<f64>::from_params(
        Type::LowPass,
        fs.hz(),
        highcut.hz(),
        Q_BUTTERWORTH_F64
    ).unwrap();

    let high_coeff = Coefficients::<f64>::from_params(
        Type::HighPass,
        fs.hz(),
        lowcut.hz(),
        Q_BUTTERWORTH_F64
    ).unwrap();

    let mut band_full = data.to_owned();
    
    for _ in 0..order {
        let low_forward = forward_filter(data, &low_coeff);
        let band_forward = forward_filter(low_forward.view(), &high_coeff);
        let low_full = backward_filter(band_forward.view(), &low_coeff);
        band_full = backward_filter(low_full.view(), &high_coeff);
    }
    
    return band_full;
}


pub fn highpass_filter(data: ArrayView1<f64>, cutoff: f64, order: usize, fs: f64) -> Array1<f64> {
    let coeff = Coefficients::<f64>::from_params(
        Type::HighPass,
        fs.hz(),
        cutoff.hz(),
        Q_BUTTERWORTH_F64
    ).unwrap();

    let mut processed_data = data.to_owned();

    for _ in 0..order {
        processed_data = forward_filter(processed_data.view(), &coeff);
        processed_data = backward_filter(processed_data.view(), &coeff);
    }

    processed_data
}

fn forward_filter(data: ArrayView1<f64>, coefficients: &Coefficients<f64>) -> Array1<f64> {
    // Create the filter instance
    let mut filter = DirectForm1::<f64>::new(*coefficients);

    // Create an owned array from the view to manipulate and return
    let mut processed_data = data.to_owned();

    // Forward pass
    for sample in processed_data.iter_mut() {
        *sample = filter.run(*sample);
    }

    processed_data
}

fn backward_filter(data: ArrayView1<f64>, coefficients: &Coefficients<f64>) -> Array1<f64> {
    // Create the filter instance
    let mut filter = DirectForm1::<f64>::new(*coefficients);

    // Create an owned array from the view to manipulate and return
    let mut processed_data = data.to_owned();

    // Reverse the data for the backward pass
    processed_data.as_slice_mut().unwrap().reverse();

    // Backward pass
    for sample in processed_data.iter_mut() {
        *sample = filter.run(*sample);
    }

    // Re-reverse the data to restore original order
    processed_data.as_slice_mut().unwrap().reverse();

    processed_data
}

/// Estimates the lower envelope of a given signal.
/// The lower envelope is defined as the minimum value within a window around each sample.
/// The window size is determined by the `window_size` parameter.

// TODO: make me more performant
pub fn lower_envelope_est(data: &Array1<f64>, window_size: usize) -> Array1<f64> {
    let mut envelope = Array1::<f64>::zeros(data.len());

    // Ensure the window size is odd to maintain symmetry around the center
    let window_size = if window_size % 2 == 0 { window_size + 1 } else { window_size };
    let half_window = window_size / 2;

    for i in 0..data.len() {
        let start = if i >= half_window { i - half_window } else { 0 };
        let end = if i + half_window < data.len() { i + half_window } else { data.len() - 1 };

        // Calculate the minimum in the current window
        envelope[i] = *data.slice(s![start..=end]).min().unwrap();
    }

    envelope
}
