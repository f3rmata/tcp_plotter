pub mod tcp_receiver {
    use serde::{Deserialize, Serialize};
    use std::{
        error::Error,
        net::{Ipv4Addr, SocketAddr},
    };
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::{TcpListener, TcpStream},
    };

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Cordinate {
        x: f64,
        y: f64,
    }

    impl Cordinate {
        pub fn new(x: f64, y: f64) -> Cordinate {
            Cordinate { x, y }
        }
    }

    async fn open_socket(
        server_ip: Ipv4Addr,
        server_port: u16,
    ) -> Result<(TcpStream, SocketAddr), Box<dyn Error>> {
        // TODO: added ipv6 support
        let listener = TcpListener::bind((server_ip, server_port)).await?;
        println!("Server listening on port {server_port}");
        let (socket, addr) = listener.accept().await?;

        Ok((socket, addr))
    }

    async fn handle_connection(
        mut socket: TcpStream,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn Error>> {
        let mut reader = BufReader::new(&mut socket);
        let mut json_data = String::new();

        // 读取数据直到遇到换行符（推荐JSON每行一条数据）
        reader.read_line(&mut json_data).await?;

        // 解析JSON数据
        let cordinate_data: Cordinate = serde_json::from_str(&json_data)?;
        println!("Received from {}: {:?}", addr, cordinate_data);

        // 发送响应
        socket
            .write_all(b"Cordinates received successfully\n")
            .await?;

        Ok(())
    }

    pub async fn tcp_server(server_ip: Ipv4Addr, server_port: u16) -> Result<(), Box<dyn Error>> {
        let (socket, addr) = open_socket(server_ip, server_port).await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, addr).await {
                eprintln!("Error handling connection: {}", e);
            }
        });

        Ok(())
    }
}

pub mod plot {
    use std::error::Error;
    use slint::{Rgb8Pixel, SharedPixelBuffer};
    use plotters::{prelude::*, style::full_palette::BLUEGREY};

    pub fn call_plotter(pixel_buffer: &mut SharedPixelBuffer<Rgb8Pixel>) -> Result<(), Box<dyn Error>>{
        // TODO: add error handling rather than panic.
        let foreground = RGBAColor(255, 255, 255, 0.8);
        let background = RGBAColor(40, 40, 40, 1.0);
    
        let size = (pixel_buffer.width(), pixel_buffer.height());
        let backend = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), size);
    
        let root = backend.into_drawing_area();
        root.fill(&background).expect("error filling drawing area");
        let mut chart = ChartBuilder::on(&root)
            .margin(20)
            .x_label_area_size(80)
            .y_label_area_size(80)
            .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)
            .expect("error building chart...");
    
        chart
            .configure_mesh()
            .axis_style(&foreground)
            .label_style(("sans-serif", 20).into_font().color(&foreground))
            .bold_line_style(&foreground.mix(0.15))
            .light_line_style(&foreground.mix(0.05))
            .draw()
            .expect("error drawing...");
    
        chart
            .draw_series(LineSeries::new(
                (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
                ShapeStyle::from(&BLUEGREY).stroke_width(3),
            ))
            .expect("error drawing series...")
            .label("y = x^2")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUEGREY));
    
        chart
            .configure_series_labels()
            .background_style(&background.mix(0.8))
            .border_style(&WHITE)
            .label_font(("sans-serif", 25).with_color(&foreground))
            .draw()
            .expect("error configuring...");
    
        root.present().expect("error presenting");
        drop(chart);
        drop(root);
        
        Ok(())
    }
}