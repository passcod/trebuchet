#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(clippy::stutter)]
#![allow(clippy::non_ascii_literal)]

#[macro_use]
extern crate serde_derive;
#[cfg_attr(test, macro_use)]
extern crate serde_json;

use crate::proto::*;
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
            Resource::NetworkBelong(NetReq::Subnet(sub)) => {
                self.belonging_ips()?.iter().any(|ip| sub.contains(ip))
            }
            _ => false,
        })
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

    let memcon1 =
        proto::Constraint::required(proto::Resource::Memory(proto::MemoryReq::Absolute(10_240)));
    println!(
        "\n{:?}\nPasses: {}",
        memcon1,
        sys.check_resource(&memcon1.resource).unwrap()
    );

    let memcon2 = proto::Constraint::required(proto::Resource::Memory(proto::MemoryReq::Absolute(
        102_400_000,
    )));
    println!(
        "\n{:?}\nPasses: {}",
        memcon2,
        sys.check_resource(&memcon2.resource).unwrap()
    );

    let cpucon1 = proto::Constraint::required(proto::Resource::Cpu(proto::CpuReq::Load(10_f32)));
    println!(
        "\n{:?}\nPasses: {}",
        cpucon1,
        sys.check_resource(&cpucon1.resource).unwrap()
    );

    let cpucon2 = proto::Constraint::required(proto::Resource::Cpu(proto::CpuReq::Load(89_f32)));
    println!(
        "\n{:?}\nPasses: {}",
        cpucon2,
        sys.check_resource(&cpucon2.resource).unwrap()
    );

    let ipcon1 = proto::Constraint::required(proto::Resource::NetworkBelong(proto::NetReq::IP(
        "::1/128".parse().unwrap(),
    )));
    println!(
        "\n{:?}\nPasses: {}",
        ipcon1,
        sys.check_resource(&ipcon1.resource).unwrap()
    );

    let ipcon2 = proto::Constraint::required(proto::Resource::NetworkBelong(proto::NetReq::IP(
        "2038::1/128".parse().unwrap(),
    )));
    println!(
        "\n{:?}\nPasses: {}",
        ipcon2,
        sys.check_resource(&ipcon2.resource).unwrap()
    );

    let subcon1 = proto::Constraint::required(proto::Resource::NetworkBelong(
        proto::NetReq::Subnet("10.0.100.0/24".parse().unwrap()),
    ));
    println!(
        "\n{:?}\nPasses: {}",
        subcon1,
        sys.check_resource(&subcon1.resource).unwrap()
    );

    let subcon2 = proto::Constraint::required(proto::Resource::NetworkBelong(
        proto::NetReq::Subnet("192.0.2.0/24".parse().unwrap()),
    ));
    println!(
        "\n{:?}\nPasses: {}",
        subcon2,
        sys.check_resource(&subcon2.resource).unwrap()
    );
}
