pub mod tcp_receiver {
    use serde::{Deserialize, Serialize};
    use std::{
        error::Error,
        net::{Ipv4Addr, SocketAddr},
        str::FromStr,
        sync::Arc,
    };
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::{TcpListener, TcpStream},
        sync::Mutex,
    };

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Cordinate {
        pub x: f64,
        pub y: f64,
    }

    slint::slint! {
        export { MainWindow } from "ui/app-window.slint";
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
        cords_clr: Arc<tokio::sync::Notify>,
        stop_token: tokio_util::sync::CancellationToken,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // TODO: Rewrite this.

        let cordinates: Arc<Mutex<Vec<Cordinate>>> = Arc::new(Mutex::new(Vec::new()));
        let server_lock = Arc::clone(&cordinates);
        let tx_clone = tx.clone();
        let slint_future = async_compat::Compat::new(async move {
            tokio::select! {
                _ = stop_token.cancelled() => {
                    println!("Server stopped.\n========");
                    return;
                },
                _ = async {
                    let mut cord_recv = server_lock.lock().await;
                    let server = TcpListener::bind((server_ip, listen_port)).await.unwrap();
                    println!("Server listening on port {server_ip}:{listen_port}");
                    loop {
                        let (mut socket, addr) = server.accept().await.unwrap();
                        socket
                            .write(format!("hello! {addr:?}\n").as_bytes())
                            .await
                            .unwrap();

                        match handle_connection(socket, addr).await {
                            Ok(cord_option) => {
                                *cord_recv = match cord_option {
                                    Some(cord) => cord,
                                    None => continue,
                                };
                            }
                            Err(e) => return e,
                        };
                        println!("{:?}", cord_recv);
                        tx_clone.send(cord_recv.clone()).unwrap();
                    }
                } => {}
            }
            // slint::quit_event_loop().unwrap();
        });

        // let clr_lock = Arc::clone(&cordinates);
        let tx_clone = tx.clone();
        slint::spawn_local(async_compat::Compat::new(async move {
            loop {
                cords_clr.notified().await;
                // let mut cords = clr_lock.lock().await;
                // cords.clear();
                // tx_clone.send(cords.clone()).unwrap();
                tx_clone.send(vec![Cordinate { x: 0.0, y: 0.0 }]).unwrap();
                println!("Cleared cordinates.");
            }
        }))?;

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
            .build_cartesian_2d(0f64..1f64, 0f64..1f64)?;

        chart
            .configure_mesh()
            .axis_style(&foreground)
            .label_style(("sans-serif", 20).into_font().color(&foreground))
            .bold_line_style(&foreground.mix(0.15))
            .light_line_style(&foreground.mix(0.05))
            .draw()?;

        chart
            .draw_series(
                data_series
                    .iter()
                    .map(|x| Circle::new((x.x, x.y), 1.5, BLUEGREY.filled())),
            )?
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
