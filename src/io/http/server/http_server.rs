

pub struct HttpServer {
    port: u16,
}


impl HttpServer {
    
    pub fn create() -> HttpServer {
        return HttpServer {
            port: 8080
        };
    } 

    pub fn get_port(&self) -> u16 {
        self.port
    }

}