use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

pub mod TcpReceiver {
    use std::{error::Error, net::Ipv4Addr};

    struct Cordinate {
        x: f64,
        y: f64,
    }
    
    impl Cordinate {
        fn new(x: f64, y: f64) -> Cordinate {
            Cordinate { x, y }
        }
    }
    
    #[tokio::main]
    async fn receive_cordinates(client_ip: String, client_port: u16) -> Result<Cordinate, Box<dyn Error>> {
        let point = Cordinate::new(0.0, 0.0);
        
        // TODO: added ipv6 support
        let ip_addr = client_ip.parse::<Ipv4Addr>().is_err();
        let listener = TcpListener::bind("127.0.0.1:8080").await?;
        
        Ok(point)
    }
}