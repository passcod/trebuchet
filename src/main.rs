#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(clippy::stutter)]
#![allow(clippy::non_ascii_literal)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[cfg_attr(test, macro_use)]
extern crate serde_json;

use crate::proto::*;
use crate::system::*;

pub mod proto;
pub mod system;

fn main() {
    println!("🌈 Hello, wonderful world!\n");

    let sys = System::new();
    println!("IPs: {:?}", sys.belonging_ips().unwrap());
    println!("Hostname: {:?}", sys.hostname());
    println!("GPU detection enabled: {:?}", sys.detect_gpu);
    println!("OpenGL available: {:?}", sys.has_opengl());
    println!("OpenCL available: {:?}", sys.has_opencl());
    println!(
        "Memory available: {:?}%",
        sys.available_memory_percent().unwrap()
    );
    println!("Load available: {:?}%", sys.available_load().unwrap());

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

    let gpucon1 = Resource::Gpu(GpuKind::OpenGL);
    println!(
        "\n{:?}\nPasses: {}",
        gpucon1,
        sys.check_resource(&gpucon1).unwrap()
    );

    let gpucon2 = Resource::Gpu(GpuKind::OpenCL);
    println!(
        "\n{:?}\nPasses: {}",
        gpucon2,
        sys.check_resource(&gpucon2).unwrap()
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

    let cons4 = vec![
        Constraint::optional(Resource::NetworkBelong(NetReq::Name("kare-kun".into()))),
        Constraint::optional(Resource::NetworkBelong(NetReq::Subnet(
            "10.0.100.0/24".parse().unwrap(),
        ))),
    ];
    println!("\n{:?}\nScores: {:?}", cons4, sys.check_constraints(&cons4));
}
