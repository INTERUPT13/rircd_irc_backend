use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

// connection handler events coming "IN" -> to the Connection handler
#[derive(Debug)]
pub struct ConnectHandlerEventIn {
}

#[derive(Debug)]
pub struct ConnectHandlerEventInResponse {
}


// connection handler events coming "Out" -> to the EndpointBackend handler
#[derive(Debug)]
pub struct ConnectHandlerEventOut {
}

#[derive(Debug)]
pub struct ConnectHandlerEventOutResponse {
}




pub struct IrcEndpointBackendConnectHandler {
    pub connect_out_events_to_endpoint_handler: mpsc::Sender<(ConnectHandlerEventOut, mpsc::Sender<ConnectHandlerEventOutResponse>)>,
    pub connect_in_events_from_endpoint_handler: mpsc::Receiver<(ConnectHandlerEventIn, mpsc::Sender<ConnectHandlerEventInResponse>)>,
    pub client_connection: TcpStream,
}

impl IrcEndpointBackendConnectHandler {
    pub async fn handle(mut self) {
        let mut msg = [0;512 + 4096];
        loop {
            tokio::select! {
                res = self.client_connection.read(&mut msg) => {
                    let bytes_read = match res {
                        Ok(bytes_read) => bytes_read,
                        Err(e) => {
                            //TODO log_debug!("couldn't read from connection socket")
                            continue;
                        },
                    };

                    
                }
            }
        }
    }
}
