use slint::{ComponentHandle, SharedPixelBuffer};
use std::error::Error;
use std::sync::Arc;
use tcp_plotter::plot::call_plotter;
use tcp_plotter::tcp_receiver::*;
// use tcp_plotter::tcp_receiver::{tcp_server, Cordinate};

slint::slint! {
    export { MainWindow, ErrorDialog } from "ui/app-window.slint";
}

fn main() -> Result<(), Box<dyn Error>> {
    let main_window = MainWindow::new()?;
    slint::set_xdg_app_id(main_window.get_appid())?;

    let (tx_e, rx_e) = std::sync::mpsc::channel::<Arc<Box<dyn Error + Send + Sync>>>();
    let (tx_cord, rx_cord) = std::sync::mpsc::channel::<Vec<Cordinate>>();

    // add ref count for callback function here.
    let main_window_weak = main_window.as_weak();
    let tx_e_server = tx_e.clone();
    main_window.on_tcp_server(move |server_ip_i, listen_port_i, pressed| {
        if !pressed {
            let ui = main_window_weak.clone();
            let (server_ip, listen_port) = match parse_socket(server_ip_i, listen_port_i) {
                Ok((ip, port)) => (ip, port),
                Err(e) => {
                    tx_e_server.send(Arc::new(e)).unwrap();
                    slint::invoke_from_event_loop(move || {
                        ui.upgrade().unwrap().set_pressed(false);
                        ui.upgrade().unwrap().window().request_redraw();
                    })
                    .unwrap();
                    return;
                }
            };

            let tx_cord_clone = tx_cord.clone();
            let tx_e_clone = tx_e_server.clone();
            slint::spawn_local(async_compat::Compat::new(async move {
                match tcp_server(server_ip, listen_port, tx_cord_clone) {
                    Ok(_) => {}
                    Err(e) => {
                        tx_e_clone.send(Arc::new(e)).unwrap();
                        return;
                    }
                }
            }))
            .unwrap();

            slint::invoke_from_event_loop(move || {
                ui.upgrade()
                    .unwrap()
                    .set_console("Server starting...".into());
                ui.upgrade().unwrap().window().request_redraw();
            })
            .unwrap();
        } else {
            // TODO: add gracefully stop notify here.
        }
    });

    // let image_model = Arc::new(Mutex::new(slint::Image::default()));
    // let image_model_clone = image_model.clone();
    let main_window_weak = main_window.as_weak();
    let tx_e_plot = tx_e.clone();
    let _plot_thread = std::thread::spawn(move || {
        loop {
            let cord = match rx_cord.try_recv() {
                Ok(c) => c,
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    continue;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    break;
                }
            };
            println!("received {:?}", cord);

            let mut pixel_buffer = SharedPixelBuffer::new(1440, 960);
            match call_plotter(&mut pixel_buffer, cord) {
                Ok(_) => {}
                Err(e) => {
                    tx_e_plot.send(Arc::new(e)).unwrap();
                    continue;
                }
            };
            //*image_model_clone.lock().unwrap() = image;

            let ui_clone: slint::Weak<MainWindow> = main_window_weak.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_clone.upgrade() {
                    let image = slint::Image::from_rgb8(pixel_buffer);
                    ui.set_plot_process(1.0);
                    ui.set_plot(image);
                    ui.window().request_redraw();
                };
            })
            .unwrap();
        }
    });

    let main_window_weak = main_window.as_weak();
    let _error_thread = std::thread::spawn(move || {
        loop {
            let catched_error = match rx_e.try_recv() {
                Ok(e) => e,
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    continue;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    break;
                }
            };

            let error_mesg = format!("{catched_error}");
            println!("{}", error_mesg);

            let ui = main_window_weak.clone();
            ui.upgrade_in_event_loop(move |ui| {
                ui.set_console(error_mesg.into());
            })
            .unwrap()
            // let error_dialog = ErrorDialog::new().unwrap();
            // error_dialog.set_error_mesg(format!("{}", catched_error).into());
            // error_dialog.run().unwrap();
        }
    });

    main_window.run()?;
    Ok(())
}
