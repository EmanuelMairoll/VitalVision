#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{self, Read};
    use ndarray::Array1;
    use crate::analysis::ppg;


    #[test]
    fn test_load_and_analyze_signal() {
        //let file_path = "../signal/VitalVision-signal-playground/notebooks/data/PPG3 02.05.2024 at 11:26.bin";
        let file_path = "../signal/VitalVision-signal-playground/notebooks/data/PPG2 18.04.2024 at 15:35 stripped.bin";

        let signal = load_signal(file_path).expect("Failed to load signal");
        let params = ppg::Parameters {
            sampling_frequency: 30.0,
            filter_cutoff_low: 1.0,
            filter_cutoff_high: 10.0,
            filter_order: 4,
            envelope_range: 23, // 0.666 seconds
            amplitude_min: 10,
            amplitude_max: 2000,
        };

        let analyzer = ppg::Analysis { params };

        // Run the analysis function
        let results = analyzer.analyze(signal);

        // Example assertion (adjust according to your result handling)
        //assert!(!results.hr_estimate.is_empty(), "Heart rate estimates should not be empty");
        println!("{:?}", results.signal_quality);
        assert!(!results.signal_quality.is_empty(), "Signal quality results should not be empty");
    }

    fn load_signal(file_path: &str) -> io::Result<Array1<u16>> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();

        // Read the entire file
        file.read_to_end(&mut buffer)?;

        // Convert bytes to u16 values assuming little-endian byte order
        let u16_data: Vec<u16> = buffer.chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        // Convert Vec<u16> to ndarray::Array1<u16>
        Ok(Array1::from_vec(u16_data))
    }
}
