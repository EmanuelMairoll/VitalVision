#[cfg(test)]
pub(crate) mod tests {
    use std::fs::File;
    use std::io::{self, Read};
    use ndarray::{Array1, ArrayView1};
    use crate::analysis::{ecg, ppg};
    use plotters::prelude::*;
    use slog::{Logger, o, Drain, info};
    use slog_async::Async;
    use slog_term::{FullFormat, TermDecorator};

    #[test]
    fn test_ppg() {
        let file_path = "ppg.bin";

        let logger = get_logger();

        let signal = load_signal_u16(file_path).expect("Failed to load signal");
        let params = ppg::Parameters {
            sampling_frequency: 30.0,
            filter_cutoff_low: 1.0,
            filter_cutoff_high: 10.0,
            filter_order: 4,
            envelope_range: 23, // 0.666 seconds
            amplitude_min: 10.0,
            amplitude_max: 2000.0,
            trough_depth_min: -0.25,
            trough_depth_max: 0.25,
            pulse_width_min: 0.333, // 200 bpm
            pulse_width_max: 1.5, // 40 bpm
        };

        let analyzer = ppg::Analysis { params, logger: logger.clone(), plotter: Some(Box::new(plot_signal)) };
        let signal = signal.map(|&x| (x as f64));

        // Run the analysis function
        let results = analyzer.analyze_view(signal.view());

        assert!(results.is_some(), "Signal quality results should not be None");
        assert!(results.unwrap().signal_quality > 0.5, "Signal quality should be greater than 0.0");
    }

    fn get_logger() -> Logger {
        let decorator = TermDecorator::new().build();
        let drain = FullFormat::new(decorator)
            .use_utc_timestamp()  // Use UTC timestamp
            .use_original_order() // Maintain the order of log fields as declared
            .build()
            .fuse();
        let async_drain = Async::new(drain).build().fuse();
        let logger = Logger::root(async_drain, o!("component" => "VVCore", "module" => "analysis-tests"));
        logger
    }

    #[test]
    fn test_ecg() {
        let file_path = "ecg.bin";

        let logger = get_logger();

        let signal = load_signal_f64(file_path).expect("Failed to load signal");
        let params = ecg::Parameters {
            sampling_frequency: 32.0,
            filter_cutoff_low: 0.6,
            filter_order: 1,
            r_peak_prominence_mad_multiple: 12.0,
            r_peak_distance: 5,
            r_peak_plateau: 0,
            hr_min: 40.0,
            hr_max: 200.0,
            hr_max_diff: 20.0,
        };

        let analyzer = ecg::Analysis { params, logger: logger.clone(), plotter: Some(Box::new(plot_signal)) };

        // Run the analysis function
        let results = analyzer.analyze_view(signal.view());

        info!(logger, "Heart rate results"; "results" => format!("{:?}", results.hr_estimate));
        info!(logger, "Signal quality results"; "results" => format!("{:?}", results.signal_quality));

        assert!(results.hr_estimate > 40.0, "Heart rate estimates should not be empty");
        assert!(results.signal_quality > 0.5, "Signal quality results should not be empty");

    }

    fn load_signal_u16(file_path: &str) -> io::Result<Array1<u16>> {
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

    fn load_signal_f64(file_path: &str) -> io::Result<Array1<f64>> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();

        // Read the entire file
        file.read_to_end(&mut buffer)?;

        // Convert bytes to u16 values assuming little-endian byte order
        let f64_data: Vec<f64> = buffer.chunks_exact(8)
            .map(|chunk| f64::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3],
                chunk[4], chunk[5], chunk[6], chunk[7]
            ]))
            .collect();

        // Convert Vec<u16> to ndarray::Array1<u16>
        Ok(Array1::from_vec(f64_data))
    }


    pub fn plot_signal(
        data: ArrayView1<f64>,
        title: &str,
        file_path: &str,
        points: Option<Vec<usize>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(file_path, (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_value = *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0f64);
        let min_value = *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0f64);

        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 40).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0..data.len() as i32, min_value..max_value)?;

        chart.configure_mesh().draw()?;

        chart.draw_series(LineSeries::new(
            data.iter().enumerate().map(|(x, y)| (x as i32, *y)),
            &RED,
        ))?;

        if let Some(indexes) = points {
            chart.draw_series(indexes.into_iter().filter_map(|index| {
                data.get(index).map(|&value| Circle::new((index as i32, value), 5, &BLUE))
            }))?;
        }

        root.present()?;
        Ok(())
    }
}
