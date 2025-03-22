use slint::SharedPixelBuffer;
use std::error::Error;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::thread;
use tcp_plotter::plot::call_plotter;
use tcp_plotter::tcp_receiver::*;
// use tcp_plotter::tcp_receiver::{tcp_server, Cordinate};

slint::slint! {
    export { MainWindow } from "ui/app-window.slint";
}

fn get_plot() -> slint::Image {
    let mut pixel_buffer = SharedPixelBuffer::new(1440, 960);
    call_plotter(&mut pixel_buffer).expect("plot failed.");
    slint::Image::from_rgb8(pixel_buffer)
}

fn start_server(server_ip: slint::SharedString, listen_port: i32, _pressed: bool) {
    let server_ip: Ipv4Addr =
        Ipv4Addr::from_str(server_ip.as_str()).expect("Parse ipv4 addr failed!");
    let listen_port: u16 = listen_port.try_into().expect("Port is not valid!");

    tcp_server(server_ip, listen_port).unwrap();
    // slint::run_event_loop_until_quit().unwrap();
}

// fn server_controller(server_ip: slint::SharedString, listen_port: i32, pressed: bool) {}

fn main() -> Result<(), Box<dyn Error>> {
    let main_window = MainWindow::new()?;
    slint::set_xdg_app_id(main_window.get_appid())?;

    // main_window.on_tcp_server(server);
    main_window.on_tcp_server(start_server);
    main_window.on_render_plot(get_plot);

    thread::spawn(move || {});

    main_window.run()?;

    Ok(())
}
