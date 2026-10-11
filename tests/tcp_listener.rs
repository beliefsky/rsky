use rsky::net::TcpListener;
use rsky::net::TcpStream;
use rsky::os::net::Shutdown;
use rsky::os::net::SockAddr;
use rsky::runtime;
use std::io;
use std::net::SocketAddr;

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
        let res = compio::net::TcpListener::bind("127.0.0.1:3000").await;
        match res {
            Ok(_) => {
                println!("服务器已启动，监听在 127.0.0.1:3000");
            }
            Err(e) => {
                println!("服务器启动失败：{}", e);
            }
        }
    });
}

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

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (mut client, addr) = listener.accept().await?;
        let addr = addr.as_addr().unwrap();
        println!("------accept------> {}:{}", addr.ip(), addr.port(),);
        runtime::spawn(async move {
            if let Err(e) = test_rsky_conn(&mut client).await {
                println!("conn error: {}", e);
            }
            // take需保持移步执行，由于此处SharedFd在该异步任务后续未使用可以这样执行
            let _ = client.close().await;
        })
        .detach();
    }
}

async fn test_rsky_conn(conn: &mut TcpStream) -> io::Result<()> {
    let mut read_buf = Vec::with_capacity(4096);
    loop {
        let (res, buf) = conn.read(read_buf).await.into();
        let n = res?;
        if n == 0 {
            break;
        }
        // let received = String::from_utf8_lossy(&buf[..n]);
        // println!("收到数据字节数: {}", n);
        read_buf = buf;

        let content = "hello world!";
        let a = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n{}",
            content.len(),
            content
        );

        let (res, _) = conn.write(a).await.into();
        res?;
        // let n = res?;
        // println!("发送数据字节数：{}", n);
    }

    conn.shutdown(Shutdown::BOTH).await
}
