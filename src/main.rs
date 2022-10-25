#[macro_use]
extern crate log;
extern crate simplelog;
use simplelog::*;
extern crate clap;

#[macro_use]
extern crate lazy_static;

use clap::{App, Arg};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net;

use std::fs::File;

// ----------------------------------------------------------------------------------------------------------
// route server -> client
async fn server_event(
    mut client_stream: net::tcp::WriteHalf<'_>,
    mut server_stream: net::tcp::ReadHalf<'_>,
) {
    loop {
        let mut recv_buf: [u8; 65536] = [0; 65536];
        let n = server_stream.read(&mut recv_buf).await;
        let n = match n {
            Ok(x) => x,
            Err(e) => {
                error!("Failed to read TCP stream: {}", e);
                client_stream.shutdown().await;
                return;
            }
        };
        if n == 0 {
            client_stream.shutdown().await;
            return;
        }
        let mess = &recv_buf[0..n];
        debug!("server -> client {:?}", mess);
        client_stream.write_all(mess).await;
    }
}

// ----------------------------------------------------------------------------------------------------------
// route client -> server
async fn client_event(
    mut client_stream: net::tcp::ReadHalf<'_>,
    mut server_stream: net::tcp::WriteHalf<'_>,
) {
    loop {
        let mut recv_buf: [u8; 65536] = [0; 65536];
        let n = client_stream.read(&mut recv_buf).await;
        let n = match n {
            Ok(x) => x,
            Err(e) => {
                error!("Failed to read TCP stream: {}", e);
                server_stream.shutdown().await;
                return;
            }
        };
        if n == 0 {
            server_stream.shutdown().await;
            return;
        }
        let mess = &recv_buf[0..n];
        debug!("client -> server {:?}", mess);
        server_stream.write_all(mess).await;
    }
}

#[tokio::main]
async fn main() {
    // ------------------------------------------------------
    // get the command line arguments
    lazy_static! {
        static ref matches: clap::ArgMatches<'static> = {
            let b = App::new("railroute")
                .version("1.0")
                .author("UE2020")
                .about("A simple (but powerful!) TCP router")
                .arg(
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .value_name("PORT")
                        .help("Sets a custom port (default is 3000)")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("address")
                        .short("a")
                        .long("address")
                        .value_name("ADDRESS")
                        .help("Sets the routed address")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("log-packets")
                        .short("l")
                        .long("log-packets")
                        .value_name("LOG-PACKETS")
                        .help("Sets the file where packets are logged.")
                        .takes_value(true),
                )
                .get_matches();
            b
        };
        static ref address: &'static str = matches.value_of("address").unwrap();
    }

    // ---------------------------------------------------------------------------
    // logging
    let log_packets = matches.value_of("log-packets").unwrap_or("");
    if log_packets == "" {
        CombinedLogger::init(vec![TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
        )])
        .unwrap();
    } else {
        CombinedLogger::init(vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed),
            WriteLogger::new(
                LevelFilter::Debug,
                Config::default(),
                File::create(log_packets).unwrap(),
            ),
        ])
        .unwrap();
    }

    // -------------------------------------------------------------------
    // get the port
    let port = matches.value_of("port").unwrap_or("3000").parse::<u64>();
    let port = match port {
        Ok(x) => x,
        Err(e) => {
            error!("Please supply a valid port: {}", e);
            return;
        }
    };

    // --------------------------------------------------------------------------
    // start listening
    let listener = net::TcpListener::bind(format!("0.0.0.0:{}", port)).await;
    let listener = match listener {
        Ok(x) => x,
        Err(e) => {
            error!("Failed to bind to address: {}", e);
            return;
        }
    };
    info!("Server listening on port {}", port);
    loop {
        // --------------------------------------------------------------------
        // accept the connection
        let (mut client_stream, _) = match listener.accept().await {
            Ok(x) => {
                info!("Accepted connection from {}", x.0.peer_addr().unwrap());
                x
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
                continue;
            }
        };

        // ------------------------------------------------------------------------
        // launch a new thread
        tokio::spawn(async move {
            let mut server_stream = match net::TcpStream::connect(String::from(*address)).await {
                Ok(x) => x,
                Err(e) => {
                    error!("Failed to connect to server: {}", e);
                    return;
                }
            };
            let addr = client_stream.peer_addr().unwrap();
            let (client_read, client_write) = client_stream.split();
            let (server_read, server_write) = server_stream.split();

            tokio::join!(
                server_event(client_write, server_read),
                client_event(client_read, server_write)
            );
            info!("Connection from {} terminated", addr);
        });
    }

    // -----------------
    // just in case?
    drop(listener);
}
