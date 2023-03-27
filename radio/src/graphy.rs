 use num_complex::Complex;

 use plotters::prelude::*;

 pub fn graph(file_name: &str, arr: Vec<Complex<f32>>) -> Result<(), Box<dyn std::error::Error>> {
     let root = BitMapBackend::new(file_name, (640, 480)).into_drawing_area();
     root.fill(&WHITE)?;
     let mut chart = ChartBuilder::on(&root)
         .caption("test", ("sans-serif", 50).into_font())
         .margin(5)
         .x_label_area_size(30)
         .y_label_area_size(30)
         .build_cartesian_2d(0f32..arr.len() as f32, -1f32..1f32)?;

     chart.configure_mesh().draw()?;

     chart
         .draw_series(LineSeries::new(
             (0..arr.len()).map(|x| x as f32).map(|x| {
                 (
                     x,
                     20.0 * ((arr.get(x as usize).unwrap().re.powf(2.0)
                         + arr.get(x as usize).unwrap().im.powf(2.0))
                     .sqrt()),
                 )
             }),
             &RED,
         ))?
         .label("y = x^2")
         .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

     chart
         .configure_series_labels()
         .background_style(&WHITE.mix(0.8))
         .border_style(&BLACK)
         .draw()?;

     root.present()?;

     Ok(())
 }

 pub fn graph_vec(file_name: &str, arr: Vec<f32>) -> Result<(), Box<dyn std::error::Error>> {
     let root = BitMapBackend::new(file_name, (640, 480)).into_drawing_area();
     root.fill(&WHITE)?;
     let mut chart = ChartBuilder::on(&root)
         .caption("test", ("sans-serif", 50).into_font())
         .margin(5)
         .x_label_area_size(30)
         .y_label_area_size(30)
         .build_cartesian_2d(0f32..arr.len() as f32, -1f32..1f32)?;

     chart.configure_mesh().draw()?;

     chart
         .draw_series(LineSeries::new(
             (0..arr.len())
                 .map(|x| x as f32)
                 .map(|x| (x, *arr.get(x as usize).unwrap())),
             &RED,
         ))?
         .label("y = x^2")
         .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

     chart
         .configure_series_labels()
         .background_style(&WHITE.mix(0.8))
         .border_style(&BLACK)
         .draw()?;

     root.present()?;

     Ok(())
 }
