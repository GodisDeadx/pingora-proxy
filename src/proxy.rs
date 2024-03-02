use async_trait::async_trait;
use log::info;
use prometheus::register_int_counter;
use structopt::StructOpt;

use pingora_core::server::configuration::Opt;
use pingora_core::server::Server;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_core::Result;
use pingora_http::ResponseHeader;
use pingora_proxy::{ProxyHttp, Session};

fn check_login(req: &pingora_http::RequestHeader) -> bool {
    req.headers.get("Authorization").map(|v| v.as_bytes()) == Some(b"passwor")
}

pub struct Gateway {
    req_metric: prometheus::IntCounter,
}

#[async_trait]
impl ProxyHttp for Gateway {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool> {
        // if session.req_header().uri.path().starts_with("/login")
        // //&& !check_login(session.req_header())
        // {
        //     let _ = session.respond_error(403).await;
        //     // return as the response is written
        //     return Ok(true);
        // }
        Ok(false)
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let addr = if session.req_header().uri.path().starts_with("/files") {
            ("files.nullptr.quest", 443)
        } else {
            ("1.1.1.1", 443)
        };

        log::info!("connecting to {addr:?}");

        let peer = Box::new(HttpPeer::new(addr, true, "one.one.one.one".to_string()));
        Ok(peer)
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        // replace any existng header
        upstream_response
            .insert_header("Server", "Gateway")
            .unwrap();

        // doesnt support h3
        upstream_response.remove_header("alt-svc");
        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora_core::Error>,
        _ctx: &mut Self::CTX,
    ) {
        let response_code = session
            .response_written()
            .map_or(0, |resp| resp.status.as_u16());
        info!(
            "{} response code: {response_code}",
            self.request_summary(session, _ctx)
        );

        self.req_metric.inc();
    }
}

pub fn run(opt: Opt) {
    env_logger::init();

    // read any cli args
    let mut server = Server::new(Some(opt)).unwrap();
    server.bootstrap();

    let mut proxy = pingora_proxy::http_proxy_service(
        &server.configuration,
        Gateway {
            req_metric: register_int_counter!("reg_counter", "Number of requests").unwrap(),
        },
    );
    proxy.add_tcp("0.0.0.0:6188");
    server.add_service(proxy);

    let mut prometheus_service_http =
        pingora_core::services::listening::Service::prometheus_http_service();
    prometheus_service_http.add_tcp("0.0.0.0:6189");
    server.add_service(prometheus_service_http);

    server.run_forever();
}
