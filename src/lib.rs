// mod arch;
pub mod buf;
pub mod driver;
pub mod os;
pub mod runtime;
pub mod thread;

#[cfg(test)]
mod tests {
    use crate::buf;
    use crate::buf::IntoInner;
    use crate::driver::op;
    use crate::os::net::Protocol;
    use crate::os::net::SockAddr;
    use crate::os::net::SocketType;
    use crate::runtime;
    use std::io;
    use std::net::SocketAddr;
    use std::os::fd::AsRawFd;

    use compio::io::AsyncRead;

    pub struct TestSync {
        name: &'static str,
        num: i32,
    }

    impl std::future::Future for TestSync {
        type Output = ();

        fn poll(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            let num = self.num;
            println!("====={}======> {}", self.name, num);

            self.get_mut().num += 1;

            if num > 4 {
                std::task::Poll::Ready(())
            } else {
                std::task::Poll::Pending
            }
        }
    }

    async fn test_rsky_ser() -> io::Result<()> {
        let addr = SockAddr::from(SocketAddr::from(([127, 0, 0, 1], 3000)));

        let socket = {
            let buf::BufResult(res, op) = runtime::submit(op::CreateSocket::new(
                addr.family(),
                SocketType::STREAM,
                Some(Protocol::TCP),
            ))
            .await;
            res?;
            op.into_inner()
        };
        let socket = std::rc::Rc::new(socket);

        {
            let buf::BufResult(res, _) = runtime::submit(op::Bind::new(socket.clone(), addr)).await;
            res?;
        }
        {
            let buf::BufResult(res, _) =
                runtime::submit(op::Listen::new(socket.clone(), 128)).await;
            res?;
        }

        loop {
            let client = {
                let buf::BufResult(res, op) =
                    runtime::submit(op::Accept::new(socket.clone())).await;
                res?;
                let (client, _) = op.into_inner();
                std::rc::Rc::new(client)
            };

            runtime::spawn(async {
                if let Err(e) = test_rsky_conn(client).await {
                    println!("conn error: {}", e);
                }
                {
                    // let fd = client.take();
                    // drop(client);
                    // let fd = fd.unwrap();
                    // let buf::BufResult(res, _) = runtime::submit(op::CloseSocket::new(fd)).await;
                    // res?;
                }
            })
            .detach();
        }

        Ok(())
    }

    async fn test_rsky_conn(conn: std::rc::Rc<std::os::fd::OwnedFd>) -> io::Result<()> {
        println!("---------> {}", conn.as_raw_fd());
        Ok(())
    }

    #[test]
    fn test_rsky() -> io::Result<()> {
        let runtime = runtime::Runtime::new()?;
        println!("hello world");

        runtime.block_on(async {
            println!("main");

            if let Err(e) = test_rsky_ser().await {
                println!("io error: {}", e);
            }

            // runtime::spawn(async {
            //     TestSync { name: "t2", num: 0 }.await;
            // })
            // .detach();
            // runtime::spawn(async {
            //     TestSync { name: "t3", num: 0 }.await;
            // })
            // .detach();

            println!("result => xxxxxxxxxxxxx");

            // TestSync { name: "t1", num: 0 }.await;
        });
        println!("exit");
        Ok(())
    }

    #[test]
    fn test_compio() {
        compio::runtime::Runtime::new().unwrap().block_on(async {
            tcp_listener().await.unwrap();
        });
    }

    async fn tcp_listener() -> std::io::Result<()> {
        let listener = compio::net::TcpListener::bind("127.0.0.1:3000").await?;
        println!("服务器已启动，监听在 127.0.0.1:3000");
        loop {
            let (mut stream, addr) = listener.accept().await?;
            println!("新客户端连接: {}", addr);
            compio::runtime::spawn(async move {
                if let Err(e) = tcp_handle(&mut stream).await {
                    eprintln!("处理客户端 {} 时出错: {}", addr, e);
                }
                println!("客户端 {} 断开连接", addr);
            })
            .detach();
        }

        Ok(())
    }

    async fn tcp_handle(stream: &mut compio::net::TcpStream) -> std::io::Result<()> {
        let mut read_buf = vec![0; 1024];
        loop {
            let (n, buf) = stream.read(read_buf).await.unwrap();
            if n == 0 {
                return Ok(());
            }
            let received = String::from_utf8_lossy(&buf[..n]);
            println!("收到数据: {}", received);

            read_buf = buf;
        }
    }
}
