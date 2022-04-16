mod http;

use tokio::net::{TcpListener,TcpStream};
use simple_log::{debug, error};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    simple_log::quick!();
    console_subscriber::init();

    let listener = TcpListener::bind("127.0.0.1:8080").await.expect("Unable to bind to socket");
    
    loop{
        
        let stream = listener.accept().await;
        match stream {
            Ok((stream,_addr)) => {
                tokio::spawn(handle_connection(stream));
            },
            Err(e) => error!("{}", e)
        }

    }
}

async fn handle_connection(conn: TcpStream) {
    simple_log::info!("Recieved connection from {}", conn.peer_addr().unwrap());

    let mut connection = http::Connection::new(conn);
    let request = connection.get_request().await;

    debug!("{:?}", request);
    
    let mut response = Vec::from("HTTP/1.1 200 OK\n\n".as_bytes());
    response.append(&mut Vec::from("hello world".as_bytes()));

    let _ = connection.send_response(http::Response{}).await;
    // connection terminated
}