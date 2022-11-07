mod thread_pool;
pub mod utils;

use std::{
    net::{TcpStream, TcpListener},
    io::{Read, Write},
    thread,
    time::Duration,
    fs::File
};

use {
    utils::types::ServerError,
    utils::consts::{BUF_SIZE, TCP_ADDRESS_PORT},
    thread_pool::ThreadPool,
};


fn handle_connection(mut stream: TcpStream) -> Result<(), ServerError> {
    let mut buff = [0u8; BUF_SIZE];
    stream.read(&mut buff)?;

    let request_status = {
        let mut end_of_reqest_status = BUF_SIZE;
        for pos in 0..BUF_SIZE {
            if buff[pos]   == b"\r"[0]    // on Linux: 13
            && buff[pos+1] == b"\n"[0] {  // on Linux: 10
                if pos == 0 {
                    return Err(
                        ServerError::BadRequest(format!(
                            "HTTP request starts with '\\r'. request: {}",
                            String::from_utf8_lossy(&buff)
                        ))
                    )
                }
                end_of_reqest_status = pos - 1;
                break;
            }
        }
        if end_of_reqest_status == BUF_SIZE {
            return Err(
                ServerError::BadRequest(format!(
                    "HTTP request doesn't contain any valid rewuest status. request: {}",
                    String::from_utf8_lossy(&buff)
                ))
            )
        }
        &buff[..=end_of_reqest_status]
    };

    let (response_status, response_body_file_path) = match request_status {
        b"GET / HTTP/1.1" => ("HTTP/1.1 200 OK\r\n\r\n",        "./templates/hello.html"),
        b"GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK\r\n\r\n", "./templates/sleep.html")
        },
        _ =>                 ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "./templates/404.html")
    };
    let response_body = {
        let mut content = String::with_capacity(512);
        File::open(&response_body_file_path)?.read_to_string(&mut content)?;
        content
    };

    stream.write(response_status.as_bytes())?;
    stream.write(response_body.as_bytes())?;
    stream.flush()?;

    Ok(())
}

pub(super) fn run(thread_pool_size: usize) -> Result<(), ServerError> {
    let pool = ThreadPool::new(thread_pool_size)?;
    let listener = TcpListener::bind(TCP_ADDRESS_PORT)?;

    for stream in listener.incoming() {
        let stream = stream?;
        pool.execute(|| {
            match handle_connection(stream) {
                Ok(_) => (),
                Err(error) => println!("{:?}", error),
            }
        })?
    }

    Ok(())
}