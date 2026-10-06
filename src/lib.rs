// mod arch;
pub mod buf;
pub mod driver;
pub mod runtime;
pub mod thread;

#[cfg(test)]
mod tests {
    use crate::driver;
    use crate::runtime;
    use std::io;

    use compio::io::AsyncRead;

    pub struct TestSync {
        name: &'static str,
        num: i32,
    }

    pub struct TestOp {}

    unsafe impl driver::OpCode for TestOp {
        type Control = usize;

        fn create_entry(&mut self, a: &mut Self::Control) -> driver::OpEntry {
            println!("========> {}", a);
            *a = 2;
            io_uring::opcode::Socket::new(2, 1, 6).build().into()
        }
        unsafe fn set_result(
            &mut self,
            a: &mut Self::Control,
            size: &io::Result<usize>,
            _: &driver::Extra,
        ) {
            println!("===2====> {}", a);

            match size {
                Ok(size) => println!("result: {}", size),
                Err(e) => println!("result error: {}", e),
            }
        }
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

    #[test]
    fn test_rsky() -> io::Result<()> {
        let runtime = runtime::Runtime::new()?;
        println!("hello world");

        runtime.block_on(async {
            println!("main");

            // runtime::spawn(async {
            //     TestSync { name: "t2", num: 0 }.await;
            // })
            // .detach();
            // runtime::spawn(async {
            //     TestSync { name: "t3", num: 0 }.await;
            // })
            // .detach();

            let a = runtime::submit(TestOp {}).await;

            println!("result => xxxxxxxxxxxxx");

            TestSync { name: "t1", num: 0 }.await;
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
