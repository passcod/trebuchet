#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use armstrong::proto::*;
use ipnet::IpNet;
use serde_json::json;

fn out<T: serde::Serialize + std::fmt::Debug>(arg: T) {
    eprintln!("{:?}", arg);
    println!("{}", json!(arg));
}

fn reqout(resource: Resource) {
    let last = std::env::args().last().unwrap();
    out(Constraint {
        resource,
        optional: last == "req",
    });
}

fn net_ip(ip: String) -> NetReq {
    let auto = ip.parse::<IpNet>();
    let v6 = format!("{}/128", ip).parse::<IpNet>();
    let v4 = format!("{}/32", ip).parse::<IpNet>();
    NetReq::IP(auto.or(v6).or(v4).expect("an ip"))
}

fn mem(num: String) -> MemoryReq {
    if num.ends_with("%") {
        let n = num.len() - 1;
        let num = &num[0..n];
        MemoryReq::Percentage(num.parse().expect("a percent"))
    } else {
        MemoryReq::Absolute(num.parse().expect("a number"))
    }
}

fn main() {
    armstrong::init();

    let mut args = std::env::args();
    args.next(); // discard arg zero

    let kind: &str = &args.next().expect("at least one arg");
    // eprintln!("kind: {}", kind);

    match kind {
        "net-ip" => {
            out(net_ip(args.next().expect("an ip")));
        }
        "net-name" => {
            let name = args.next().expect("an hostname");
            out(NetReq::Name(name));
        }
        "net-subnet" => {
            let sub = args.next().expect("a subnet");
            out(NetReq::Subnet(sub.parse().expect("a subnet")));
        }
        "gpu" => match args.next().expect("a gpu kind").as_str() {
            "cuda" => out(GpuKind::CUDA),
            "open-gl" => out(GpuKind::OpenGL),
            "open-cl" => out(GpuKind::OpenCL),
            _ => println!("uh?"),
        },
        "cpu" => {
            let load = args.next().expect("a number");
            out(CpuReq::Load(load.parse().expect("a number")));
        }
        "mem" => {
            out(mem(args.next().expect("a number (kb) or percent%")));
        }
        "resource" => match args.next().expect("a resource name").as_str() {
            "access-ip" => reqout(Resource::NetworkAccess(net_ip(args.next().expect("an ip")))),
            "belong-ip" => reqout(Resource::NetworkBelong(net_ip(args.next().expect("an ip")))),
            "access-name" => reqout(Resource::NetworkAccess(NetReq::Name(
                args.next().expect("a name"),
            ))),
            "belong-name" => reqout(Resource::NetworkBelong(NetReq::Name(
                args.next().expect("a name"),
            ))),
            "access-subnet" => reqout(Resource::NetworkAccess(NetReq::Subnet(
                args.next().and_then(|s| s.parse().ok()).expect("a subnet"),
            ))),
            "belong-subnet" => reqout(Resource::NetworkBelong(NetReq::Subnet(
                args.next().and_then(|s| s.parse().ok()).expect("a subnet"),
            ))),
            "gpu" => reqout(Resource::Gpu(
                match args.next().expect("a gpu kind").as_str() {
                    "cuda" => GpuKind::CUDA,
                    "open-gl" => GpuKind::OpenGL,
                    "open-cl" => GpuKind::OpenCL,
                    _ => return println!("uh?"),
                },
            )),
            "cpu" => reqout(Resource::Cpu(CpuReq::Load(
                args.next().and_then(|s| s.parse().ok()).expect("a number"),
            ))),
            "mem" => reqout(Resource::Memory(mem(args.next().expect("a number")))),
            _ => println!("uh?"),
        },
        _ => println!("don’t know about that one chief"),
    };
}
