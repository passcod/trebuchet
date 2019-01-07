#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Debug;

fn main() {
    println!("Hello, world!");
    
    check(Constraint::required(Resource::Memory(MemoryReq::Absolute(128))));
    check(Constraint::optional(Resource::Memory(MemoryReq::Percentage(50))));
    check(Constraint::required(Resource::Cpu(CpuReq::Percentage(5))));
    check(Constraint::required(Resource::NetworkAccess(NetReq::Subnet("10.0.0.0/8".into()))));
    check(Constraint::required(Resource::NetworkBelong(NetReq::IP("172.0.2.81".into()))));
    
    uncheck::<Constraint>(r#"{"optional":false,"resource":{"memory":200}}"#);
    uncheck::<Constraint>(r#"{"optional":false,"resource":{"memory":"64b"}}"#);
    uncheck::<Constraint>(r#"{"optional":false,"resource":{"memory":"157575b"}}"#);
    uncheck::<Constraint>(r#"{"optional":false,"resource":{"memory":"35m"}}"#);
    uncheck::<Constraint>(r#"{"optional":false,"resource":{"memory":"2%"}}"#);
    
    uncheck::<Constraint>(r#"{"optional":true,"resource":{"network-access":{"subnet":"10.0.100.0/24"}}}"#);
}

fn check<T: Debug + Serialize>(strct: T) {
    println!("-> {:?}\n== {}\n\n", strct, json!(strct));
}

fn uncheck<T: Debug + Deserialize<'static>>(unstr: &'static str) {
    let strct: T = serde_json::from_str(unstr).unwrap();
    println!("-> {}\n== {:?}\n\n", unstr, strct);
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Worker {
    name: String,
    inputs: Vec<DataDef>,
    outputs: Vec<DataDef>,
    constraints: Vec<Constraint>,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
struct DataDef {
    name: String,
    optional: bool,
    datatype: DataType,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
enum DataType {
    Stream,
    Bool,
    Int,
    Float,
    String,
    Binary,
    Json,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Constraint {
    resource: Resource,
    optional: bool, // will be scheduled on matching nodes preferentially, but can run on non-matching nodes
}

impl Constraint {
    pub fn required(resource: Resource) -> Self {
        Self { resource, optional: false }
    }
    
    pub fn optional(resource: Resource) -> Self {
        Self { resource, optional: true }
    }
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Resource {
    Memory(MemoryReq), // in kb
    Cpu(CpuReq), // in abstract units
    Gpu(GpuKind), // not the brand or power, more the tech interface available
    NetworkBelong(NetReq),
    NetworkAccess(NetReq),
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
enum MemoryReq {
    #[serde(deserialize_with = "kb_from_strum")]
    Absolute(usize),

    #[serde(deserialize_with = "percentage_from_string")]
    #[serde(serialize_with = "percentage_to_string")]
    Percentage(u16),
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
enum CpuReq {
    Absolute(usize),

    #[serde(deserialize_with = "percentage_from_string")]
    #[serde(serialize_with = "percentage_to_string")]
    Percentage(u16),
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
fn kb_from_strum<'de, D>(d: D) -> Result<(usize), D::Error> where D: Deserializer<'de> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Strum {
        Number(usize),
        String(String),
    }
    
    match Strum::deserialize(d)? {
        Strum::Number(u) => Ok(u),
        Strum::String(s) => if s.ends_with(vec!['b', 'k', 'm', 'g', 't'].as_slice()) {
            let n = s.len() - 1;
            let (num, unit) = s.split_at(n);
            num.parse::<usize>().map_err(|_err| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Str(num), &"a string representation of a usize")
            }).map(|n| match unit {
                "b" => ((n as f64) / 1024.0).max(1.0) as usize,
                "k" => n,
                "m" => n * 1024,
                "g" => n * 1024 * 1024,
                "t" => n * 1024 * 1024 * 1024,
                _ => unreachable!()
            })
        } else {
            Err(serde::de::Error::invalid_type(serde::de::Unexpected::Str(&s), &"a number of bytes with a unit letter b/k/m/g/t"))
        }
    }
}

fn percentage_from_string<'de, D>(d: D) -> Result<(u16), D::Error> where D: Deserializer<'de> {
    let pc = String::deserialize(d)?;
    if pc.ends_with('%') {
        let n = pc.len() - 1;
        let num = &pc[0..n];
        num.parse::<u16>().map_err(|_err| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Str(num), &"a string representation of a u16")
        })
    } else {
        Err(serde::de::Error::invalid_type(serde::de::Unexpected::Str(&pc), &"a percentage"))
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn percentage_to_string<S>(pc: &u16, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
    s.serialize_str(&format!("{}%", pc))
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
enum GpuKind {
    CUDA,
    
    #[serde(rename = "open-cl")]
    OpenCL,
    
    #[serde(rename = "open-gl")]
    OpenGL,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
enum NetReq {
    #[serde(rename = "ip")]
    IP(String), // belong: has this ip; access: can ping this ip
    Name(String), // belong: has this hostname; access: can resolve & ping this hostname
    Subnet(String), // belong: has ip within this subnet; access: can route to this subnet
}
