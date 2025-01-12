//! hermes documentation

use std::env;
use std::net::Ipv4Addr;
use std::sync::Arc;

use getopts::Options;

use atlas::dns::context::{ResolveStrategy, ServerContext};
use atlas::dns::protocol::{DnsRecord, TransientTtl};
use atlas::dns::server::{DnsServer, DnsTcpServer, DnsUdpServer};
use atlas::web::server::WebServer;

use simple_logger::SimpleLogger;

use log::LevelFilter;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {

    SimpleLogger::new().with_colors(true).init().unwrap();


    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag(
        "a",
        "authority",
        "disable support for recursive lookups, and serve only local zones",
    );
    opts.optopt(
        "f",
        "forward",
        "forward replies to specified dns server",
        "SERVER",
    );
    opts.optopt("p", "port", "listen on specified port", "PORT");

    let opt_matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if opt_matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let mut context = Arc::new(ServerContext::new());

    if let Some(ctx) = Arc::get_mut(&mut context) {
        let mut index_rootservers = true;
        if opt_matches.opt_present("f") {
            match opt_matches
                .opt_str("f")
                .and_then(|x| x.parse::<Ipv4Addr>().ok())
            {
                Some(ip) => {
                    ctx.resolve_strategy = ResolveStrategy::Forward {
                        host: ip.to_string(),
                        port: 53,
                    };
                    index_rootservers = false;
                    log::info!("Running as forwarder");
                }
                None => {
                    log::info!("Forward parameter must be a valid Ipv4 address");
                    return;
                }
            }
        }

        if opt_matches.opt_present("a") {
            ctx.allow_recursive = false;
        }

        if opt_matches.opt_present("p") {
            match opt_matches.opt_str("p").and_then(|x| x.parse::<u16>().ok()) {
                Some(port) => {
                    ctx.dns_port = port;
                }
                None => {
                    log::info!("Port parameter must be a valid port");
                    return;
                }
            }
        }

        match ctx.initialize() {
            Ok(_) => {}
            Err(e) => {
                log::info!("Server failed to initialize: {:?}", e);
                return;
            }
        }

        if index_rootservers {
            let _ = ctx.cache.store(&get_rootservers());
        }
    }

    log::info!("Listening on port {}", context.dns_port);

    // Start DNS servers
    if context.enable_udp {
        let udp_server = DnsUdpServer::new(context.clone(), 20);
        if let Err(e) = udp_server.run_server() {
            log::info!("Failed to bind UDP listener: {:?}", e);
        }
    }

    if context.enable_tcp {
        let tcp_server = DnsTcpServer::new(context.clone(), 20);
        if let Err(e) = tcp_server.run_server() {
            log::info!("Failed to bind TCP listener: {:?}", e);
        }
    }

    // Start web server
    if context.enable_api {
        let webserver = WebServer::new(context.clone());
        webserver.run_webserver(true);
    }
}

fn get_rootservers() -> Vec<DnsRecord> {
    let mut rootservers = Vec::new();

    rootservers.push(DnsRecord::Ns {
        domain: "".to_string(),
        host: "a.root-servers.net".to_string(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::A {
        domain: "a.root-servers.net".to_string(),
        addr: "198.41.0.4".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::Aaaa {
        domain: "a.root-servers.net".to_string(),
        addr: "2001:503:ba3e::2:30".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });

    rootservers.push(DnsRecord::Ns {
        domain: "".to_string(),
        host: "b.root-servers.net".to_string(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::A {
        domain: "b.root-servers.net".to_string(),
        addr: "192.228.79.201".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::Aaaa {
        domain: "b.root-servers.net".to_string(),
        addr: "2001:500:84::b".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });

    rootservers.push(DnsRecord::Ns {
        domain: "".to_string(),
        host: "c.root-servers.net".to_string(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::A {
        domain: "c.root-servers.net".to_string(),
        addr: "192.33.4.12".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::Aaaa {
        domain: "c.root-servers.net".to_string(),
        addr: "2001:500:2::c".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });

    rootservers.push(DnsRecord::Ns {
        domain: "".to_string(),
        host: "d.root-servers.net".to_string(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::A {
        domain: "d.root-servers.net".to_string(),
        addr: "199.7.91.13".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::Aaaa {
        domain: "d.root-servers.net".to_string(),
        addr: "2001:500:2d::d".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });

    rootservers.push(DnsRecord::Ns {
        domain: "".to_string(),
        host: "e.root-servers.net".to_string(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::A {
        domain: "e.root-servers.net".to_string(),
        addr: "192.203.230.10".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });

    rootservers.push(DnsRecord::Ns {
        domain: "".to_string(),
        host: "f.root-servers.net".to_string(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::A {
        domain: "f.root-servers.net".to_string(),
        addr: "192.5.5.241".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::Aaaa {
        domain: "f.root-servers.net".to_string(),
        addr: "2001:500:2f::f".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });

    rootservers.push(DnsRecord::Ns {
        domain: "".to_string(),
        host: "g.root-servers.net".to_string(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::A {
        domain: "g.root-servers.net".to_string(),
        addr: "192.112.36.4".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });

    rootservers.push(DnsRecord::Ns {
        domain: "".to_string(),
        host: "h.root-servers.net".to_string(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::A {
        domain: "h.root-servers.net".to_string(),
        addr: "198.97.190.53".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::Aaaa {
        domain: "h.root-servers.net".to_string(),
        addr: "2001:500:1::53".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });

    rootservers.push(DnsRecord::Ns {
        domain: "".to_string(),
        host: "i.root-servers.net".to_string(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::A {
        domain: "i.root-servers.net".to_string(),
        addr: "192.36.148.17".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::Aaaa {
        domain: "i.root-servers.net".to_string(),
        addr: "2001:7fe::53".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });

    rootservers.push(DnsRecord::Ns {
        domain: "".to_string(),
        host: "j.root-servers.net".to_string(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::A {
        domain: "j.root-servers.net".to_string(),
        addr: "192.58.128.30".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::Aaaa {
        domain: "j.root-servers.net".to_string(),
        addr: "2001:503:c27::2:30".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });

    rootservers.push(DnsRecord::Ns {
        domain: "".to_string(),
        host: "k.root-servers.net".to_string(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::A {
        domain: "k.root-servers.net".to_string(),
        addr: "193.0.14.129".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::Aaaa {
        domain: "k.root-servers.net".to_string(),
        addr: "2001:7fd::1".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });

    rootservers.push(DnsRecord::Ns {
        domain: "".to_string(),
        host: "l.root-servers.net".to_string(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::A {
        domain: "l.root-servers.net".to_string(),
        addr: "199.7.83.42".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::Aaaa {
        domain: "l.root-servers.net".to_string(),
        addr: "2001:500:3::42".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });

    rootservers.push(DnsRecord::Ns {
        domain: "".to_string(),
        host: "m.root-servers.net".to_string(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::A {
        domain: "m.root-servers.net".to_string(),
        addr: "202.12.27.33".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });
    rootservers.push(DnsRecord::Aaaa {
        domain: "m.root-servers.net".to_string(),
        addr: "2001:dc3::35".parse().unwrap(),
        ttl: TransientTtl(3600000),
    });

    rootservers
}
