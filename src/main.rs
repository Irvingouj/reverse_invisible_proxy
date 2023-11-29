use clap::builder::FalseyValueParser;
use clap::{Arg, ArgAction, Command};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::str::FromStr;
use tokio::net::TcpStream;
use tracing::{Level, info, trace};

#[tokio::main]
async fn main() {
    let matches = Command::new("My CLI Program")
        .version("1.0")
        .author("Your Name")
        .about("Handles custom arguments")
        .arg(
            Arg::new("upstream")
                .short('u')
                .long("upstream")
                .value_name("URL")
                .help("Sets a custom upstream URL")
                .required(true),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Sets a custom port")
                .default_value("8080")
                .required(false),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .required(false)
                .action(ArgAction::SetTrue)
                .value_parser(FalseyValueParser::new()),
        )
        .get_matches();

    let binding = matches.clone();
    let verbose = binding.get_flag("verbose");
    println!("verbose: {}", verbose);
    match verbose {
        true => {
            tracing_subscriber::fmt()
                .with_span_events(FmtSpan::FULL)
                .with_max_level(Level::TRACE)
                .init()
        }
        false => {
            tracing_subscriber::fmt().with_env_filter(EnvFilter::default()).init()
        }
    };

    info!("Starting trace" );

    let upstream_arg: &String = binding.get_one("upstream").unwrap();
    let port: &String = matches.get_one("port").unwrap();
    let addr = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port.parse::<u16>().unwrap());
    // info all the args
    info!("upstream: {}", upstream_arg);
    info!("port: {}", port);
    info!("addr: {}", addr);
    
    let upstream_to_socket_result = upstream_arg.to_socket_addrs();
    let upstream_ips = upstream_to_socket_result.unwrap().collect::<Vec<SocketAddr>>();

    // create tcp listener at addr
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    while let Ok((inbound, addr)) = listener.accept().await {
        trace!("received connection from {:?}", addr);
        let upstream = upstream_ips.get(0).unwrap().clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(inbound, upstream).await {
                eprintln!("an error occurred; error = {:?}", e);
            }
        });
    }
}

async fn handle_connection(
    mut inbound: TcpStream,
    upstream: SocketAddr,
) -> Result<(), &'static str> {
    // create tcp connection to upstream
    let mut outbound = TcpStream::connect(upstream).await.unwrap();
    tokio::io::copy_bidirectional(&mut inbound, &mut outbound)
        .await
        .map_err(|_| "copy failed")?;
    Ok(())
}
