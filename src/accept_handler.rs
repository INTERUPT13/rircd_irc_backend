use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use std::sync::Arc;

use tokio::net::TcpListener;
use std::net::SocketAddr;
use std::collections::HashMap;
use crate::connect_handler::ConnectHandlerEventInResponse;
use crate::connect_handler::IrcEndpointBackendConnectHandler;
use crate::connect_handler::ConnectHandlerEventOutResponse;
use crate::connect_handler::ConnectHandlerEventOut;
use crate::connect_handler::ConnectHandlerEventIn;

#[derive(Debug)]
pub struct AcceptHandlerEventInResponse {
}
#[derive(Debug)]
pub struct AcceptHandlerEventIn {
}

#[derive(Debug)]
pub struct AcceptHandlerEventOutResponse {
}
#[derive(Debug)]
pub struct AcceptHandlerEventOut {
}

// meant to be passed to the accept handlers. these senders provie a way for the connect/accept
// handlers to send messages to the endpoin handler
pub struct IrcEndpointBackendAcceptHandler {

    // used by the accept handler to send events to the endpoint handler
    pub accept_out_events_to_endpoint_handler: mpsc::Sender<(AcceptHandlerEventOut, mpsc::Sender<AcceptHandlerEventOutResponse>)>,
    // used by the connect  handler to send events to the endpoint handler
    pub connect_out_events_to_endpoint_handler: mpsc::Sender<(ConnectHandlerEventOut, mpsc::Sender<ConnectHandlerEventOutResponse>)>,


    
    // the accept handler uses this to receive events from the endpoint handler
    pub accept_in_events_from_endpoint_handler: mpsc::Receiver<(AcceptHandlerEventIn, mpsc::Sender<AcceptHandlerEventInResponse>)>,

    // on every accepted connection the accept handler:
    // - creates a channel
    //  -and passes the receiver end to the connect handler it spawns
    //  -and saves the sender end to this hashmap with the accepted connections remote end
    //  (cliets ip addr) as the key for the sender
    //
    // since this structure is shared with the endpoint handler. The endpoint handler can then
    // lookup the sender associated with a given connection handler to send messages to it
    // ( or just list the connections its aware of by listing the keys)
    pub connect_handlers: Arc<RwLock<HashMap<SocketAddr, mpsc::Sender<(ConnectHandlerEventIn, mpsc::Sender<ConnectHandlerEventInResponse>)>>>>,

    pub listener: TcpListener,
}


impl IrcEndpointBackendAcceptHandler {
    pub async fn handle(self) {

        loop {
            tokio::select! {
                client_connection = self.listener.accept() =>  {
                    let (client_connection, client_addr) = match client_connection {
                        Ok((addr, conn)) => (addr, conn),
                        Err(e) => {
                            //log_info!("couldn't accept incomming connection {}", e);
                            continue;
                        },
                    };
                    let (connect_in_events_to_connection_handler, connect_in_events_from_endpoint_handler) = mpsc::channel(99); // TODO cfg

                    self.connect_handlers.write().await.insert(client_addr, connect_in_events_to_connection_handler);

                    let conn_handler =  IrcEndpointBackendConnectHandler {
                        connect_out_events_to_endpoint_handler: self.connect_out_events_to_endpoint_handler.clone(),
                        connect_in_events_from_endpoint_handler,
                        client_connection,
                    };

                    tokio::spawn( conn_handler.handle() );
                }
            }
        }
    }
}
