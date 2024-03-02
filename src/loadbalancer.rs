use async_trait::async_trait;
use pingora::prelude::*;
use std::sync::Arc;

//use crate::json_handling;

pub struct LB(Arc<LoadBalancer<RoundRobin>>);

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();

    fn new_ctx(&self) -> () {
        ()
    }

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self
            .0
            .select(b"", 256) // has does not matter for RoundRobin
            .unwrap();

        println!("Upstream peer is: {:?}", upstream);

        // Set SNI to one.one.one.one
        let peer = Box::new(HttpPeer::new(
            upstream,
            true,
            "panel.nullptr.quest".to_string(),
        ));
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        upstream_request
            .insert_header("Host", "panel.nullptr.quest")
            .unwrap();
        Ok(())
    }
}

pub fn run() {
    let mut server = Server::new(Some(Opt::default())).unwrap();
    server.bootstrap();

    let mut upstream = LoadBalancer::try_from_iter(["panel.nullptr.quest:443"]).unwrap();

    let hc = TcpHealthCheck::new();
    upstream.set_health_check(hc);
    upstream.health_check_frequency = Some(std::time::Duration::from_secs(1));

    let background = background_service("health check", upstream);
    let upstream = background.task();

    let mut lb = http_proxy_service(&server.configuration, LB(upstream));
    lb.add_tcp("0.0.0.0:6188");

    server.add_service(background);

    server.add_service(lb);
    server.run_forever();
}
