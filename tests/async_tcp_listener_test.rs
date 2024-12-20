#[cfg(test)]
mod test_listener {

    use futures::executor::block_on;
    use rsky::net::tcp::TcpListener;
    use std::{io, net::SocketAddr};

    async fn create_server() -> io::Result<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
        let listener = TcpListener::bind(&addr)?;

        println!("tcp server create success");
        // loop {
        //     let (stream, _) = listener.accept().await?;
        //     println!("accept new cli");
        // }

        Ok(())
    }

    #[test]
    fn it_works() {
        match block_on(create_server()) {
            Ok(_) => println!("server exit"),
            Err(e) => println!("create server error: {}", e),
        };
        println!("end");
    }
}
