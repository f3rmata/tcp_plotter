pub mod tcp_receiver {
    use std::{error::Error, net::SocketAddr};
    use serde::{Deserialize, Serialize};
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::{TcpListener, TcpStream},
    };

    #[derive(Debug, Serialize, Deserialize)]
    struct Cordinate {
        x: f64,
        y: f64,
    }
    
    impl Cordinate {
        fn new(x: f64, y: f64) -> Cordinate {
            Cordinate { x, y }
        }
    }
    
    async fn open_socket(server_ip: String, server_port: u16)
                   -> Result<(TcpStream, SocketAddr), Box<dyn Error>> {

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
        // 创建带缓冲的读取器

        let mut reader = BufReader::new(&mut socket);
        let mut json_data = String::new();

        // 读取数据直到遇到换行符（推荐JSON每行一条数据）
        reader.read_line(&mut json_data).await?;

        // 解析JSON数据
        let sensor_data: Cordinate = serde_json::from_str(&json_data)?;

        println!(
            "Received from {}: {:?}",
            addr, sensor_data
        );

        // 发送响应
        socket
            .write_all(b"Cordinates received successfully\n")
            .await?;

        Ok(())
    }
    
    #[tokio::main]
    async fn tcp_server(server_ip: String, server_port: u16) -> Result<(), Box<dyn Error>> {
        let (socket, addr) = open_socket(server_ip, server_port).await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, addr).await {
                eprintln!("Error handling connection: {}", e);
            }
        });

        Ok(())
    }
}
