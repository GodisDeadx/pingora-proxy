//mod json_handling;
mod loadbalancer;
mod proxy;

use pingora_core::server::configuration::Opt;
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    proxy::run(opt);
    loadbalancer::run();
}
