use Blinter::TcpReceiver;
use slint::SharedPixelBuffer;
use plotters::{prelude::*, style::full_palette::BLUEGREY};

slint::slint! {
    export { MainWindow } from "ui/app-window.slint";
}

fn render_plot() -> slint::Image {
    // TODO: add error handling rather than panic.
    let foreground = RGBAColor(255,255,255,0.8);
    let background = RGBAColor(40,40,40,1.0);

    let mut pixel_buffer = SharedPixelBuffer::new(1440, 960);
    let size = (pixel_buffer.width(), pixel_buffer.height()); 
    let backend = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), size);
    
    let root = backend.into_drawing_area();
    root.fill(&background).expect("error filling drawing area");
    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(80)
        .y_label_area_size(80)
        .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32).expect("error building chart...");
    
    chart.configure_mesh().axis_style(&foreground)
        .label_style(("sans-serif", 20).into_font().color(&foreground))
        .bold_line_style(&foreground.mix(0.15))
        .light_line_style(&foreground.mix(0.05))
        .draw().expect("error drawing...");
    
    chart
        .draw_series(LineSeries::new(
            (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
            ShapeStyle::from(&BLUEGREY).stroke_width(3),
        )).expect("error drawing series...")
        .label("y = x^2")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUEGREY));
    
    chart
        .configure_series_labels()
        .background_style(&background.mix(0.8))
        .border_style(&WHITE)
        .label_font(("sans-serif", 25).with_color(&foreground))
        .draw().expect("error configuring...");
    
    root.present().expect("error presenting");
    drop(chart);
    drop(root); 

    slint::Image::from_rgb8(pixel_buffer)
}

pub fn main() {
    let main_window = MainWindow::new().unwrap();
    main_window.on_render_plot(render_plot);
    main_window.run().unwrap();
}
