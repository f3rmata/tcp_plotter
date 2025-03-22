use slint::{ComponentHandle, SharedPixelBuffer};
use std::error::Error;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tcp_plotter::plot::call_plotter;
use tcp_plotter::tcp_receiver::*;
use tokio::sync::{mpsc, Notify};
// use tcp_plotter::tcp_receiver::{tcp_server, Cordinate};

slint::slint! {
    export { MainWindow } from "ui/app-window.slint";
}

fn main() -> Result<(), Box<dyn Error>> {
    let main_window = MainWindow::new()?;
    slint::set_xdg_app_id(main_window.get_appid())?;

    let start_server = Arc::new(Notify::new());
    let server_ip = Arc::new(Mutex::new(Ipv4Addr::new(127, 0, 0, 1)));
    let listen_port = Arc::new(Mutex::new(2887));
    let (tx, mut rx) = mpsc::channel::<Vec<Cordinate>>(8);

    // add ref count for callback function here.
    let server_ip_ui = server_ip.clone();
    let listen_port_ui = listen_port.clone();
    let start_server_ui = start_server.clone();
    main_window.on_tcp_server(move |server_ip_i, listen_port_i, pressed| {
        if !pressed {
            let server_ip: Ipv4Addr =
                Ipv4Addr::from_str(server_ip_i.as_str()).expect("Parse ipv4 addr failed!");
            let listen_port: u16 = listen_port_i.try_into().expect("Port is not valid!");

            let mut ip = server_ip_ui.lock().unwrap();
            let mut port = listen_port_ui.lock().unwrap();
            *ip = server_ip;
            *port = listen_port;

            start_server_ui.notify_one();
        } else {
            // TODO: add gracefully stop notify here.
        }
    });

    let server_ip_thread = server_ip.clone();
    let listen_port_thread = listen_port.clone();
    slint::spawn_local(async_compat::Compat::new(async move {
        start_server.notified().await;
        tcp_server(
            *server_ip_thread.lock().unwrap(),
            *listen_port_thread.lock().unwrap(),
            tx,
        )
        .unwrap();
    }))?;

    // let image_model = Arc::new(Mutex::new(slint::Image::default()));

    // let image_model_clone = image_model.clone();
    let main_window_weak = main_window.as_weak();
    slint::spawn_local(async_compat::Compat::new(async move {
        loop {
            let cord = rx.recv().await.unwrap();
            println!("received {:?}", cord);
    
            let mut pixel_buffer = SharedPixelBuffer::new(1440, 960);
            call_plotter(&mut pixel_buffer, cord).unwrap();
            //*image_model_clone.lock().unwrap() = image;

            let ui_clone = main_window_weak.clone();
            slint::invoke_from_event_loop( move || {
                if let Some(ui) = ui_clone.upgrade() {
                    let image = slint::Image::from_rgb8(pixel_buffer);
                    ui.set_plot_process(0.5);
                    ui.set_plot(image);
                    // ui.invoke_render_plot();
                    ui.window().request_redraw();
                };
            }).unwrap();
        }
    }))?;

    // main_window.on_render_plot(move || {
    //     println!("called render plot");
    //     // image_model.lock().unwrap().clone()
    // });

    main_window.run()?;

    Ok(())
}
