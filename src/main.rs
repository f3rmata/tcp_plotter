use slint::SharedPixelBuffer;
use std::net::TcpListener;
use std::error::Error;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::io::Write;
use tcp_plotter::plot::call_plotter;
// use tcp_plotter::tcp_receiver::{tcp_server, Cordinate};

slint::slint! {
    export { MainWindow } from "ui/app-window.slint";
}

fn render_plot() -> slint::Image {
    let mut pixel_buffer = SharedPixelBuffer::new(1440, 960);
    call_plotter(&mut pixel_buffer).expect("plot failed.");
    slint::Image::from_rgb8(pixel_buffer)
}

fn start_server(server_ip: slint::SharedString, listen_port: i32) {
    println!("Acquiring...");
    let server_ip: Ipv4Addr = Ipv4Addr::from_str(server_ip.as_str()).expect("Parse ipv4 addr failed!");
    let listen_port: u16 = listen_port.try_into().expect("Port is not valid!");

    let listener = TcpListener::bind((server_ip, listen_port)).unwrap();
    let server = std::thread::spawn(move || {
        let mut stream = listener.incoming().next().unwrap().unwrap();
        stream.write("Hello World\n".as_bytes()).unwrap();
    });

    let slint_future = async move {
    };

    slint::spawn_local(async_compat::Compat::new(slint_future)).unwrap();
    slint::run_event_loop_until_quit().unwrap();
    server.join().unwrap();
}

fn main() -> Result<(), Box<dyn Error>> {
    let main_window = MainWindow::new()?;
    slint::set_xdg_app_id(main_window.get_appid())?;

    // main_window.on_tcp_server(server);
    main_window.on_tcp_server(start_server);
    main_window.on_render_plot(render_plot);

    main_window.run()?;
    
    Ok(())
}
