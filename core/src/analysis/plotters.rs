extern crate plotters;

use ndarray::{Array1, ArrayView1};
use plotters::prelude::*;

pub fn plot_signal(data: ArrayView1<i32>, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(file_path, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_value = *data.iter().max().unwrap_or(&0);
    let min_value = *data.iter().min().unwrap_or(&0);

    let mut chart = ChartBuilder::on(&root)
        .caption("Signal Plot", ("sans-serif", 40).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..data.len() as i32, min_value..max_value)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        data.iter().enumerate().map(|(x, y)| (x as i32, *y)),
        &RED,
    ))?;

    root.present()?;
    Ok(())
}

use plotters::prelude::*;

pub fn plot_signal_f64(
    data: ArrayView1<f64>,
    file_path: &str,
    points: Option<Vec<usize>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(file_path, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_value = *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0f64);
    let min_value = *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0f64);

    let mut chart = ChartBuilder::on(&root)
        .caption("Signal Plot", ("sans-serif", 40).into_font())
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
