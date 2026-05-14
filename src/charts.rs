use plotters::prelude::*;
use crate::stream::PricePair;
use std::io::Cursor;
pub fn generate_chart_png(pair: &str, history: &[PricePair]) -> Vec<u8> {
    let width = 800;
    let height = 450;
    let mut buffer = vec![0u8; (width * height * 3) as usize];

    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&RGBColor(25, 25, 25)).unwrap(); 

        let filtered: Vec<f64> = history.iter()
            .filter(|p| !p.bid_price.is_empty() && p.bid_price[0] > 0.0)
            .map(|p| p.bid_price[0])
            .collect();

        let display_data: Vec<f64> = filtered.iter()
            .rev().take(100).rev() 
            .cloned()
            .collect();

        if display_data.is_empty() { return vec![]; }

        let min_p = display_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_p = display_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max_p - min_p;
        let padding = if range == 0.0 { 0.1 } else { range * 0.1 };

        let mut chart = ChartBuilder::on(&root)
            .caption(format!("Pair: {}", pair), ("sans-serif", 25).into_font().color(&WHITE))
            .margin(15)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(0..display_data.len(), (min_p - padding)..(max_p + padding))
            .unwrap();

		chart.configure_mesh()
			.disable_x_mesh()
			.bold_line_style(RGBColor(50, 50, 50))
			.light_line_style(RGBColor(40, 40, 40))
			.x_label_style(("sans-serif", 12).into_font().color(&WHITE))
			.y_label_style(("sans-serif", 12).into_font().color(&WHITE))
			.draw().unwrap();

        chart.draw_series(
            AreaSeries::new(
                display_data.iter().enumerate().map(|(i, &p)| (i, p)),
                min_p - padding, 
                &RGBColor(0, 150, 255).mix(0.2),
            ).border_style(RGBColor(0, 150, 255).stroke_width(2))
        ).unwrap();

        chart.draw_series(
            display_data.iter().enumerate().map(|(i, &p)| {
                Circle::new((i, p), 2, RGBColor(0, 150, 255).filled())
            })
        ).unwrap();

        root.present().unwrap();
    }

    let mut png_output = Cursor::new(Vec::new());
    image::write_buffer_with_format(
        &mut png_output, &buffer, width, height,
        image::ColorType::Rgb8, image::ImageFormat::Png,
    ).expect("Failed to encode PNG");

    png_output.into_inner()
}
