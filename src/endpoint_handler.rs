use std::collections::HashMap;
use std::sync::Arc;
use std::net::{SocketAddr, AddrParseError};
use tokio::sync::{mpsc,RwLock};
use tokio::net::{TcpListener,TcpStream};
use color_eyre::{eyre::{eyre, WrapErr}, Result};
use crate::connect_handler::IrcEndpointBackendConnectHandler;
use futures::stream::futures_unordered::FuturesUnordered;
use std::str::FromStr;
use std::pin::Pin;

use crate::accept_handler::IrcEndpointBackendAcceptHandler;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use std::time::Duration;
use tokio::time::timeout;

use futures::stream::{StreamExt,Stream};


use crate::accept_handler::{
    AcceptHandlerEventIn, AcceptHandlerEventInResponse,
    AcceptHandlerEventOut, AcceptHandlerEventOutResponse
};

use crate::connect_handler::{
    ConnectHandlerEventIn, ConnectHandlerEventInResponse,
    ConnectHandlerEventOut, ConnectHandlerEventOutResponse
};




pub struct IrcEndpointBackend {
    // the endpoint handler can use this to lookup the accept handler for a given
    // connection. It can also just enumerate the connection handlers
    accept_handlers: HashMap<SocketAddr, mpsc::Sender<(AcceptHandlerEventIn, mpsc::Sender<AcceptHandlerEventInResponse>)>>,
    
    // the endpoint handler can use this to lookup the connection handler for a given
    // connection. It can also just enumerate the connection handlers
    // - Arc<RwLock<>> is so that the acceptor thread can fill in this information
    //   when a connection is accepted while the EndpointBackend handler accesses it as well
    connect_handlers: Arc<RwLock<HashMap<SocketAddr, mpsc::Sender<(ConnectHandlerEventIn, mpsc::Sender<ConnectHandlerEventInResponse>)>>>>,

    // keeps track of the plain addresses the handler should bind to
    bind_addrs_plain: Vec<SocketAddr>,
    listeners_plain: Vec<TcpListener>,

    // keeps track of the tls addresses the handler should bind to
    bind_addrs_tls: Vec<SocketAddr>,
    //listeners_tls: Vec<TcpListener>,

}






impl IrcEndpointBackendConnectHandler {
    async fn send_event_to_endpoint_handler(&mut self,ev: ConnectHandlerEventOut) -> Result<ConnectHandlerEventOutResponse> {
        let max_time = Duration::from_secs(5); //TODO cfg

        let (connect_out_response_s, mut connect_out_response_r) = mpsc::channel(99); // TODO cfg

        self.connect_out_events_to_endpoint_handler.send( (ev, connect_out_response_s) ).await
            .wrap_err("failed to send connect event to endpoint handler")?;

        match timeout(max_time, connect_out_response_r.recv()).await {
            Err(e) => Err(eyre!("timeout {:?} reached while waiting for an response to an connect event",max_time)),
            Ok(None) => Err(eyre!("previously send ConnectHandlerEvent did not receive a response from the endpoint handler")),
            Ok(Some(r)) => Ok(r),
        }
    }
}

impl IrcEndpointBackendAcceptHandler {
    async fn a_to_endpoint_handler(&mut self,ev: AcceptHandlerEventOut) -> Result<AcceptHandlerEventOutResponse> {
        let max_time = Duration::from_secs(5); //TODO cfg

        let (accept_out_response_s, mut accept_out_response_r) = mpsc::channel(99); // TODO cfg

        self.accept_out_events_to_endpoint_handler.send( (ev,accept_out_response_s) ).await
            .wrap_err("failed to send accept event to endpoint handler")?;

        match timeout(max_time, accept_out_response_r.recv()).await {
            Err(e) => Err(eyre!("timeout {:?} reached while waiting for an response to an accept event",max_time)),
            Ok(None) => Err(eyre!("previously send AcceptHandlerEvent did not receive a response from the endpoint handler")),
            Ok(Some(r)) => Ok(r),
        }
    }
}




impl IrcEndpointBackend {
    pub fn new(bind_addrs_plain: Vec<&str>, bind_addrs_tls: Vec<&str>) -> Result<Self> {
        // TODO  if port = 0 replace it in bind_addrs with the port allocated
        if bind_addrs_tls.is_empty() && bind_addrs_plain.is_empty() {
            return Err(eyre!("no bind addrs specified for IrcEndpointBackend creation"));
        }
        Ok(Self {
            accept_handlers: HashMap::new(),
            connect_handlers: Arc::new(RwLock::new(HashMap::new())),

            bind_addrs_plain: {
                bind_addrs_plain.iter().map(|addr| addr.parse()).collect::<Result<Vec<_>,_>>()?
            },
            listeners_plain: Vec::new(),

            bind_addrs_tls: {
                bind_addrs_plain.iter().map(|addr| addr.parse()).collect::<Result<Vec<_>,_>>()?
            },
            //listeners_tls: Vec::new(),
        })
    }

    // TODO actually rather meant to be part of the EndpointBackend trait
    // -attempts to start the EndpointBackend. In our case it tries to bind
    // to the sockets and (if enabled) setup an tls handler and fails if it can't
    pub async fn start(mut self) -> Result<()> {
        
        // plain_listeners_init
        self.listeners_plain = {
            let bind_futs: FuturesUnordered<_> = self.bind_addrs_plain.iter().map(|addr| {
                TcpListener::bind(addr)
            }).collect();

            let listeners: Result<Vec<_>,_> = bind_futs.collect::<Vec<_>>().await.into_iter().collect();
            listeners?
        };
        
        // tls_listeners_init
        let _listeners_tls = {
            // ...
        };


        tokio::spawn( self.handle() );

        Ok(())
    }
    async fn handle(mut self) {
        let (accept_out_events_to_endpoint_handler,mut accept_out_events_from_accept_handler) = mpsc::channel(99); //cfg
        let (connect_out_events_to_endpoint_handler,mut connect_out_events_from_connect_handler) = mpsc::channel(99); //cfg

        // spawn off accept handlers
        self.listeners_plain.into_iter().for_each(|l| {
            let (accept_in_events_to_accept_handler, accept_in_events_from_endpoint_handler) = mpsc::channel(99); // TODO cfg

            self.accept_handlers.insert(l.local_addr().unwrap(), accept_in_events_to_accept_handler);

            let accept_shared = IrcEndpointBackendAcceptHandler {

                accept_out_events_to_endpoint_handler: accept_out_events_to_endpoint_handler.clone(),
                accept_in_events_from_endpoint_handler,

                connect_out_events_to_endpoint_handler: connect_out_events_to_endpoint_handler.clone(),

                connect_handlers: self.connect_handlers.clone(),

                listener: l,
            };
            tokio::spawn(  accept_shared.handle() );
        });

        loop {
            tokio::select! {
                ev = accept_out_events_from_accept_handler.recv() => {
                }

                ev = connect_out_events_from_connect_handler.recv() => {
                }
            }
        }
    }
}

impl IrcEndpointBackend {
    pub fn lol() {
    }
}
