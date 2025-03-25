pub mod tcp_receiver {
    use serde::{Deserialize, Serialize};
    use std::{
        error::Error,
        net::{Ipv4Addr, SocketAddr},
        str::FromStr,
    };
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::{TcpListener, TcpStream},
    };

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Cordinate {
        pub x: i32,
        pub y: f64,
    }

    async fn handle_connection(
        mut socket: TcpStream,
        addr: SocketAddr,
    ) -> Result<Option<Vec<Cordinate>>, Box<dyn Error>> {
        let mut reader = BufReader::new(&mut socket);
        let mut json_data = String::new();

        // 读取数据直到遇到换行符（推荐JSON每行一条数据）
        reader.read_line(&mut json_data).await?;

        // 解析JSON数据
        match serde_json::from_str(&json_data.trim()) {
            Ok(cord) => {
                println!("Received from {}: {:?}", addr, cord);
                socket.write(b"Cordinate received\n").await?;
                Ok(cord)
            }
            Err(e) => {
                eprintln!("{e}");
                socket
                    .write(format!("Error parsing: {e}\n").as_bytes())
                    .await?;
                Ok(None)
            }
        }
    }

    pub fn parse_socket(
        server: slint::SharedString,
        port: i32,
    ) -> Result<(Ipv4Addr, u16), Box<dyn Error + Send + Sync>> {
        let server_ip: Ipv4Addr = Ipv4Addr::from_str(server.as_str())?;
        let listen_port: u16 = port.try_into()?;

        Ok((server_ip, listen_port))
    }

    pub fn tcp_server(
        server_ip: Ipv4Addr,
        listen_port: u16,
        tx: std::sync::mpsc::Sender<Vec<Cordinate>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let slint_future = async_compat::Compat::new(async move {
            let server = TcpListener::bind((server_ip, listen_port)).await.unwrap();
            println!("Server listening on port {server_ip}:{listen_port}");
            let mut cordinates: Vec<Cordinate> = Vec::new();
            loop {
                let (mut socket, addr) = server.accept().await.unwrap();
                socket
                    .write(format!("hello! {addr:?}\n").as_bytes())
                    .await
                    .unwrap();

                match handle_connection(socket, addr).await {
                    Ok(cord_option) => {
                        match cord_option {
                            Some(cord) => cordinates = [cordinates, cord].concat(),
                            None => continue,
                        };
                    }
                    Err(e) => return e,
                };
                println!("{:?}", cordinates);
                tx.send(cordinates.clone()).unwrap();
            }
            // slint::quit_event_loop().unwrap();
        });

        println!("Start Listening...");
        let _thread_local = slint::spawn_local(slint_future)?;

        Ok(())
    }
}

pub mod plot {
    use crate::tcp_receiver::Cordinate;
    use plotters::{prelude::*, style::full_palette::BLUEGREY};
    use slint::{Rgb8Pixel, SharedPixelBuffer};
    use std::error::Error;

    pub fn call_plotter(
        pixel_buffer: &mut SharedPixelBuffer<Rgb8Pixel>,
        data_series: Vec<Cordinate>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // TODO: add error handling rather than panic.
        let foreground = RGBAColor(255, 255, 255, 0.8);
        let background = RGBAColor(40, 40, 40, 1.0);

        println!("ploting...");
        let size = (pixel_buffer.width(), pixel_buffer.height());
        let backend = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), size);

        let root = backend.into_drawing_area();
        root.fill(&background)?;
        let mut chart = ChartBuilder::on(&root)
            .margin(20)
            .x_label_area_size(80)
            .y_label_area_size(80)
            .build_cartesian_2d(0..4096, 0f64..1f64)?;

        chart
            .configure_mesh()
            .axis_style(&foreground)
            .label_style(("sans-serif", 20).into_font().color(&foreground))
            .bold_line_style(&foreground.mix(0.15))
            .light_line_style(&foreground.mix(0.05))
            .draw()?;

        chart
            .draw_series(LineSeries::new(
                data_series.iter().map(|x| (x.x, x.y)),
                ShapeStyle::from(&BLUEGREY).stroke_width(3),
            ))
            .expect("error drawing series...")
            .label("freq/amp response")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUEGREY));

        chart
            .configure_series_labels()
            .background_style(&background.mix(0.8))
            .border_style(&WHITE)
            .label_font(("sans-serif", 25).with_color(&foreground))
            .draw()?;

        root.present()?;
        drop(chart);
        drop(root);

        Ok(())
    }
}
