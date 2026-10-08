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
    use crate::os::net::RecvFlags;
    use crate::os::net::SendFlags;
    use crate::os::net::Shutdown;
    use crate::os::net::SockAddr;
    use crate::os::net::SocketType;
    use crate::runtime;
    use std::io;
    use std::mem::MaybeUninit;
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
            let (res, op) = runtime::submit(op::CreateSocket::new(
                addr.family(),
                SocketType::STREAM,
                Some(Protocol::TCP),
            ))
            .await
            .into();
            res?;
            op.into_inner()
        };
        let socket = std::rc::Rc::new(socket);

        {
            let (res, _) = runtime::submit(op::Bind::new(socket.clone(), addr))
                .await
                .into();
            res?;
        }
        {
            let (res, _) = runtime::submit(op::Listen::new(socket.clone(), 128))
                .await
                .into();
            res?;
        }

        loop {
            let client = {
                let (res, op) = runtime::submit(op::Accept::new(socket.clone()))
                    .await
                    .into();
                res?;
                let (client, _) = op.into_inner();
                std::rc::Rc::new(client)
            };

            runtime::spawn(async {
                if let Err(e) = test_rsky_conn(client).await {
                    println!("conn error: {}", e);
                }

                // let fd = client.take();
                // drop(client);
                // let fd = fd.unwrap();
                // let buf::BufResult(res, _) = runtime::submit(op::CloseSocket::new(fd)).await;
                // res?;
            })
            .detach();
        }
    }

    impl buf::IoBuf for Vec<u8> {
        fn as_init(&self) -> &[u8] {
            self.as_slice()
        }
    }
    impl buf::IoBufMut for Vec<u8> {
        fn as_uninit(&mut self) -> &mut [MaybeUninit<u8>] {
            let ptr = self.as_mut_ptr() as *mut MaybeUninit<u8>;
            let cap = self.capacity();
            unsafe { std::slice::from_raw_parts_mut(ptr, cap) }
        }
    }

    impl buf::IoBuf for String {
        fn as_init(&self) -> &[u8] {
            self.as_bytes()
        }
    }

    async fn test_rsky_conn(conn: std::rc::Rc<std::os::fd::OwnedFd>) -> io::Result<()> {
        println!("-----fd----> {}", conn.as_raw_fd());

        let mut read_buf: Vec<u8> = vec![0; 1024];
        loop {
            {
                let (res, op) =
                    runtime::submit(op::Recv::new(conn.clone(), read_buf, RecvFlags::EMPTY))
                        .await
                        .into();
                let n = res?;
                if n == 0 {
                    println!("------EOF------> {}", conn.as_raw_fd());
                    break;
                }
                let buf = op.into_inner();
                // let received = String::from_utf8_lossy(&buf[..n]);
                println!("收到数据字节数: {}", n);

                read_buf = buf;
            }

            let content = "hello world!";

            let a = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n{}",
                content.len(),
                content
            );
            let (res, _) = runtime::submit(op::Send::new(conn.clone(), a, SendFlags::EMPTY))
                .await
                .into();
            let n = res?;
            println!("发送数据字节数：{}", n);
        }

        let (res, _) = runtime::submit(op::ShutdownSocket::new(conn.clone(), Shutdown::BOTH))
            .await
            .into();
        res?;

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
