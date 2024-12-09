
pub mod io;


#[cfg(test)]
mod tests {
    use crate::io::http::server::HttpServer;

    #[test]
    fn it_works() {
    
        let server = HttpServer::create();

        println!("hello world -> {}", server.get_port());
    }
}
