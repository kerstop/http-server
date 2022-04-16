use std::collections::HashMap;

use tokio::{net::TcpStream, io::{BufStream, BufReader, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt}};
use simple_log::*;

#[derive(Debug)]
pub enum Method{
    Get,
    Post
}

#[derive(Debug)]
pub enum ParseError {
    EmptyRequest,
    MalformedRequest,
    UnrecognizedMethod,
    IOError,
}
/* 
impl std::error::Error for ParseError{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
    
    
}
*/
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptyRequest => f.write_str("no data found in frame body"),
            ParseError::MalformedRequest => f.write_str("non ascii characters found in request"),
            ParseError::UnrecognizedMethod => f.write_str("method name was not recognized"),
            ParseError::IOError => f.write_str("unable to read data")
        }
    }
}

#[derive(Debug)]
pub struct Request{
    pub method: Method,
    pub url: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct Response {

}
pub struct Connection{
    stream: BufStream<TcpStream>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream: BufStream::new(stream),
        }
    }
    
    pub async fn get_request(&mut self) -> Result<Request, ParseError> {
        let mut start_line = String::new();
        let _ = self.stream.read_line(&mut start_line).await
        .map_err(|_| ParseError::MalformedRequest)?;
        let mut fields = start_line.split_ascii_whitespace();
        
        let method = match fields.next() {
            Some("GET") => Method::Get,
            Some("POST") => Method::Post,
            Some(_) => return Err(ParseError::UnrecognizedMethod),
            None => return Err(ParseError::MalformedRequest),
        };
        
        let url = match fields.next() {
            Some(u) => u.to_string(),
            None => return Err(ParseError::MalformedRequest),
        };
        
        let version = match fields.next() {
            Some(v) => v.to_string(),
            None => return Err(ParseError::MalformedRequest),
        };

        let mut headers = HashMap::new();

        loop {
            let mut line = String::new();
            let _line_length = self.stream.read_line(&mut line)
                .await.map_err(|_| ParseError::MalformedRequest)?;
            line = line.trim_end().to_string();
            if line.is_empty() {
                break;
            }

            let (key, val) = match line.find(':') {
                Some(i) => {
                    (line[0..i].to_string(), line[i+1..].trim().to_string())
                },
                None => return Err(ParseError::MalformedRequest),
            };
            
            headers.insert(key, val);
        };
        
        let mut body:Vec<u8>;
        if let Some(body_length) =  headers.get("Content-Length"){
            match body_length.trim().parse::<usize>() {
                Ok(i) => {
                    body = vec![0;i];

                    let _bytes_read = self.stream.read_exact(body.as_mut_slice()).await.expect("Problem reading body");
                     
                },
                Err(_) => return Err(ParseError::MalformedRequest),
            }
        }
        
        Ok(Request{
            method,
            url,
            version,
            headers,
            body: None
        })
    }

    pub async fn send_response(&mut self, res: Response) -> std::io::Result<()> {
        let mut response = Vec::from("HTTP/1.1 200 OK\n\n".as_bytes());
        response.append(&mut Vec::from("hello world".as_bytes()));
       
        let _ = self.stream.write(response.as_slice()).await.expect("could not write bytes");
        self.stream.flush().await.expect("could not flush write buffer");
        Ok(())
    }
}
