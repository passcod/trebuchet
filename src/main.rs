#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(clippy::stutter)]
#![allow(clippy::non_ascii_literal)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[cfg_attr(test, macro_use)]
extern crate serde_json;

use crate::proto::*;
use hostname::get_hostname;
use ipnet::IpNet;
use std::io::Result as IoResult;
use systemstat::Platform;

pub mod proto;

struct System {
    platform: systemstat::System,
}

impl System {
    pub fn new() -> Self {
        let platform = systemstat::System::new();
        Self { platform }
    }

    pub fn available_memory(&self) -> IoResult<usize> {
        self.platform.memory().map(|mem| mem.free.as_usize() / 1024)
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn available_memory_percent(&self) -> IoResult<f32> {
        self.platform
            .memory()
            .map(|mem| (mem.free.as_usize() as f32) / (mem.total.as_usize() as f32))
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn available_load(&self) -> IoResult<f32> {
        self.platform
            .load_average()
            .map(|load| 100_f32 - (load.one / num_cpus::get() as f32) * 100_f32)
    }

    #[allow(clippy::expect_fun_call)]
    pub fn belonging_ips(&self) -> IoResult<Vec<IpNet>> {
        let mut nets = Vec::new();
        for net in self.platform.networks()?.values() {
            for addr in &net.addrs {
                nets.push(
                    match addr.addr {
                        systemstat::data::IpAddr::V4(ip) => format!("{:?}/32", ip),
                        systemstat::data::IpAddr::V6(ip) => format!("{:?}/128", ip),
                        _ => continue,
                    }.parse()
                        .expect(&format!("Malformed IP address from system {:?}", addr)),
                );
            }
        }
        Ok(nets)
    }

    pub fn check_resource(&self, res: &Resource) -> IoResult<bool> {
        Ok(match res {
            Resource::Memory(MemoryReq::Absolute(free)) => self.available_memory()? >= *free,
            Resource::Memory(MemoryReq::Percentage(available)) => {
                &self.available_memory_percent()? >= available
            }
            Resource::Cpu(CpuReq::Load(available)) => &self.available_load()? >= available,
            Resource::NetworkBelong(NetReq::IP(ip)) => self.belonging_ips()?.contains(ip),
            Resource::NetworkBelong(NetReq::Name(host)) => {
                get_hostname().map_or(false, |name| host == &name)
            }
            Resource::NetworkBelong(NetReq::Subnet(sub)) => {
                self.belonging_ips()?.iter().any(|ip| sub.contains(ip))
            }
            _ => false,
        })
    }

    /// Returns a pass and a score for this system based on the given constraints.
    ///
    /// The score is calculated by assigning one point to every constraint that resolves as a pass.
    /// If a single required constraint fails, the entire thing resolves to `None` aka "cannot run
    /// this set of constraints". But if all required constraints pass and some optional constraints
    /// fail, the score will be _higher_ for systems that fulfill more constraints than others.
    ///
    /// Constraint checks that error out will be logged but resolve as a fail instead of aborting.
    pub fn check_constraints(&self, cons: &[Constraint]) -> Option<usize> {
        let mut score = 0;

        for con in cons {
            let ok = match self.check_resource(&con.resource) {
                Ok(pass) => pass,
                Err(err) => {
                    error!("Failed to run check for {:?}: {:?}", con.resource, err);
                    false
                }
            };

            if ok {
                score += 1;
            } else if !con.optional {
                return None;
            }
        }

        Some(score)
    }
}

fn main() {
    println!("🌈 Hello, wonderful world!\n");

    let sys = System::new();
    println!(
        "Memory available: {:?} KiB",
        sys.available_memory().unwrap()
    );
    println!("IPs: {:?}", sys.belonging_ips().unwrap());
    println!("Inverse load: {:?}", sys.available_load().unwrap());
    println!("Hostname: {:?}", get_hostname());

    let memcon1 = Resource::Memory(MemoryReq::Absolute(10_240));
    println!(
        "\n{:?}\nPasses: {}",
        memcon1,
        sys.check_resource(&memcon1).unwrap()
    );

    let memcon2 = Resource::Memory(MemoryReq::Absolute(102_400_000));
    println!(
        "\n{:?}\nPasses: {}",
        memcon2,
        sys.check_resource(&memcon2).unwrap()
    );

    let cpucon1 = Resource::Cpu(CpuReq::Load(10_f32));
    println!(
        "\n{:?}\nPasses: {}",
        cpucon1,
        sys.check_resource(&cpucon1).unwrap()
    );

    let cpucon2 = Resource::Cpu(CpuReq::Load(89_f32));
    println!(
        "\n{:?}\nPasses: {}",
        cpucon2,
        sys.check_resource(&cpucon2).unwrap()
    );
    );

    let ipcon1 = Resource::NetworkBelong(NetReq::IP("::1/128".parse().unwrap()));
    println!(
        "\n{:?}\nPasses: {}",
        ipcon1,
        sys.check_resource(&ipcon1).unwrap()
    );

    let ipcon2 = Resource::NetworkBelong(NetReq::IP("2038::1/128".parse().unwrap()));
    println!(
        "\n{:?}\nPasses: {}",
        ipcon2,
        sys.check_resource(&ipcon2).unwrap()
    );

    let subcon1 = Resource::NetworkBelong(NetReq::Subnet("10.0.100.0/24".parse().unwrap()));
    println!(
        "\n{:?}\nPasses: {}",
        subcon1,
        sys.check_resource(&subcon1).unwrap()
    );

    let subcon2 = Resource::NetworkBelong(NetReq::Subnet("192.0.2.0/24".parse().unwrap()));
    println!(
        "\n{:?}\nPasses: {}",
        subcon2,
        sys.check_resource(&subcon2).unwrap()
    );

    let namecon1 = Resource::NetworkBelong(NetReq::Name("kaydel-ko".into()));
    println!(
        "\n{:?}\nPasses: {}",
        namecon1,
        sys.check_resource(&namecon1).unwrap()
    );

    let namecon2 = Resource::NetworkBelong(proto::NetReq::Name("example.com".into()));
    println!(
        "\n{:?}\nPasses: {}",
        namecon2,
        sys.check_resource(&namecon2).unwrap()
    );

    let cons1 = vec![
        Constraint::required(Resource::NetworkBelong(NetReq::Name("kaydel-ko".into()))),
        Constraint::required(Resource::NetworkBelong(NetReq::Subnet(
            "10.0.100.0/24".parse().unwrap(),
        ))),
    ];
    println!("\n{:?}\nScores: {:?}", cons1, sys.check_constraints(&cons1));

    let cons2 = vec![
        Constraint::required(Resource::NetworkBelong(NetReq::Name("kare-kun".into()))),
        Constraint::required(Resource::NetworkBelong(NetReq::Subnet(
            "10.0.100.0/24".parse().unwrap(),
        ))),
    ];
    println!("\n{:?}\nScores: {:?}", cons2, sys.check_constraints(&cons2));

    let cons3 = vec![
        Constraint::optional(Resource::NetworkBelong(NetReq::Name("kare-kun".into()))),
        Constraint::required(Resource::NetworkBelong(NetReq::Subnet(
            "10.0.100.0/24".parse().unwrap(),
        ))),
    ];
    println!("\n{:?}\nScores: {:?}", cons3, sys.check_constraints(&cons3));
}
