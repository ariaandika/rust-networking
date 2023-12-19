#![allow(unused)]

use std::io::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use httparse;

const TWO_SEC: Duration = Duration::from_secs(2);

pub fn serve() {
    let tcp = TcpListener::bind("127.0.0.1:8000").unwrap();

    let mut n = 2;

    let (mut stream, _) = tcp.accept().unwrap();

    let mut stream2 = stream.try_clone().expect("Cant clone");

    // loop {
    println!("REQ");
    // {
    //     let mut reader = BufReader::new(&mut stream);
    //     let mut buf = String::new();
    //
    //     'app: loop {
    //         match reader.read_line(&mut buf).unwrap() {
    //            0 => break 'app,
    //            2 => break 'app,
    //            n => {}
    //         }
    //     }
    //
    //     println!("Readcontent: {buf}");
    // }

    {
        let mut writer = BufWriter::new(&mut stream);
        writer.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: keep-alive\r\nKeep-Alive: timeout=4\r\n\r\nAbff").unwrap();
    }
    //     match n - 1 {
    //         0 => return,
    //         _ => n = n - 1
    //     };
    // }
}

pub fn connect() {
    let mut stream = TcpStream::connect("127.0.0.1:8000").unwrap();

    stream.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();

    {
        let mut reader = BufReader::new(&mut stream);
        let mut buf = String::new();

        reader.read_line(&mut buf).unwrap();

        print!("Response {buf}");
    }
}

pub fn core() {
    println!("OOF {:?}",std::thread::available_parallelism()); // 4
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // core()
        serve()
    }

    // #[test]
    // fn tryme() {
    //     std::thread::sleep(Duration::from_secs(1));
    //     connect()
    // }
}
