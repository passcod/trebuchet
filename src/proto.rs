use serde::{Deserialize, Deserializer, Serializer};

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Worker {
    pub name: String,
    pub inputs: Vec<DataDef>,
    pub outputs: Vec<DataDef>,
    pub constraints: Vec<Constraint>,
}

impl Worker {
    pub fn new(
        name: &str,
        inputs: Vec<DataDef>,
        outputs: Vec<DataDef>,
        constraints: Vec<Constraint>,
    ) -> Result<Self, CreateError> {
        if name.is_empty() {
            return Err(CreateError::EmptyWorkerName);
        }

        Ok(Self {
            name: name.to_string(),
            inputs,
            outputs,
            constraints,
        })
    }
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "error")]
pub enum CreateError {
    EmptyWorkerName,
    EmptyDataName,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DataDef {
    pub name: String,
    pub datatype: DataType,
    pub optional: bool,
}

impl DataDef {
    pub fn new(name: &str, datatype: DataType, optional: bool) -> Result<Self, CreateError> {
        if name.is_empty() {
            return Err(CreateError::EmptyDataName);
        }

        Ok(Self {
            name: name.to_string(),
            datatype,
            optional,
        })
    }

    pub fn required(name: &str, datatype: DataType) -> Result<Self, CreateError> {
        Self::new(name, datatype, false)
    }

    pub fn optional(name: &str, datatype: DataType) -> Result<Self, CreateError> {
        Self::new(name, datatype, true)
    }
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DataType {
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
pub struct Constraint {
    pub resource: Resource,
    pub optional: bool, // will be scheduled on matching nodes preferentially, but can run on non-matching nodes
}

impl Constraint {
    pub fn required(resource: Resource) -> Self {
        Self {
            resource,
            optional: false,
        }
    }

    pub fn optional(resource: Resource) -> Self {
        Self {
            resource,
            optional: true,
        }
    }
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Resource {
    Memory(MemoryReq), // in kb
    Cpu(CpuReq),       // in abstract units
    Gpu(GpuKind),      // not the brand or power, more the tech interface available
    NetworkBelong(NetReq),
    NetworkAccess(NetReq),
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum MemoryReq {
    #[serde(deserialize_with = "kb_from_strum")]
    Absolute(usize),

    #[serde(deserialize_with = "percentage_from_string")]
    #[serde(serialize_with = "percentage_to_string")]
    Percentage(u16),
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum CpuReq {
    Absolute(usize),

    #[serde(deserialize_with = "percentage_from_string")]
    #[serde(serialize_with = "percentage_to_string")]
    Percentage(u16),
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_sign_loss)]
fn kb_from_strum<'de, D>(d: D) -> Result<(usize), D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Strum {
        Number(usize),
        String(String),
    }

    match Strum::deserialize(d)? {
        Strum::Number(u) => Ok(u),
        Strum::String(s) => {
            if s.ends_with(vec!['b', 'k', 'm', 'g', 't'].as_slice()) {
                let n = s.len() - 1;
                let (num, unit) = s.split_at(n);
                num.parse::<usize>()
                    .map_err(|_err| {
                        serde::de::Error::invalid_value(
                            serde::de::Unexpected::Str(num),
                            &"a string representation of a usize",
                        )
                    })
                    .map(|n| match unit {
                        "b" => ((n as f64) / 1024.0).max(1.0) as usize,
                        "k" => n,
                        "m" => n * 1024,
                        "g" => n * 1024 * 1024,
                        "t" => n * 1024 * 1024 * 1024,
                        _ => unreachable!(),
                    })
            } else {
                Err(serde::de::Error::invalid_type(
                    serde::de::Unexpected::Str(&s),
                    &"a number of bytes with a unit letter b/k/m/g/t",
                ))
            }
        }
    }
}

fn percentage_from_string<'de, D>(d: D) -> Result<(u16), D::Error>
where
    D: Deserializer<'de>,
{
    let pc = String::deserialize(d)?;
    if pc.ends_with('%') {
        let n = pc.len() - 1;
        let num = &pc[0..n];
        num.parse::<u16>().map_err(|_err| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(num),
                &"a string representation of a u16",
            )
        })
    } else {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Str(&pc),
            &"a percentage",
        ))
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn percentage_to_string<S>(pc: &u16, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("{}%", pc))
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GpuKind {
    #[serde(rename = "cuda")]
    CUDA,

    #[serde(rename = "open-cl")]
    OpenCL,

    #[serde(rename = "open-gl")]
    OpenGL,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetReq {
    #[serde(rename = "ip")]
    IP(String), // belong: has this ip; access: can ping this ip
    Name(String), // belong: has this hostname; access: can resolve & ping this hostname
    Subnet(String), // belong: has ip within this subnet; access: can route to this subnet
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_assert_eq {
        ($name:ident, $left:expr, $right:expr) => {
            #[test]
            fn $name() {
                assert_eq!($left, $right);
            }
        };
    }

    #[test]
    fn impossible_empty_worker_name() {
        assert_eq!(
            Worker::new("", Vec::new(), Vec::new(), Vec::new()),
            Err(CreateError::EmptyWorkerName)
        );
    }

    #[test]
    fn impossible_empty_data_name() {
        assert_eq!(
            DataDef::new("", DataType::String, true),
            Err(CreateError::EmptyDataName)
        );
    }

    test_assert_eq!(
        encode_worker_bare,
        json!(Worker::new("bare", Vec::new(), Vec::new(), Vec::new()).unwrap()).to_string(),
        r#"{"constraints":[],"inputs":[],"name":"bare","outputs":[]}"#
    );

    test_assert_eq!(
        encode_worker_full,
        json!(Worker::new(
            "bare",
            vec![
                DataDef::required("name", DataType::String).unwrap(),
                DataDef::required("events", DataType::Stream).unwrap(),
                DataDef::optional("running", DataType::Bool).unwrap()
            ],
            vec![DataDef::required("created", DataType::Bool).unwrap()],
            vec![
                Constraint::required(Resource::Memory(MemoryReq::Absolute(50_000))),
                Constraint::optional(Resource::Cpu(CpuReq::Percentage(10)))
            ]
        )
        .unwrap())
        .to_string(),
        r#"{"constraints":[{"optional":false,"resource":{"memory":50000}},{"optional":true,"resource":{"cpu":"10%"}}],"inputs":[{"datatype":"string","name":"name","optional":false},{"datatype":"stream","name":"events","optional":false},{"datatype":"bool","name":"running","optional":true}],"name":"bare","outputs":[{"datatype":"bool","name":"created","optional":false}]}"#
    );

    test_assert_eq!(
        encode_absolute_number,
        json!(Constraint::required(Resource::Memory(MemoryReq::Absolute(
            128
        ))))
        .to_string(),
        r#"{"optional":false,"resource":{"memory":128}}"#
    );
    test_assert_eq!(
        encode_percentage_string,
        json!(Constraint::optional(Resource::Memory(
            MemoryReq::Percentage(50)
        )))
        .to_string(),
        r#"{"optional":true,"resource":{"memory":"50%"}}"#
    );
    test_assert_eq!(
        encode_percentage_gt_100,
        json!(Constraint::required(Resource::Cpu(CpuReq::Percentage(153)))).to_string(),
        r#"{"optional":false,"resource":{"cpu":"153%"}}"#
    );
    test_assert_eq!(
        encode_keys_kebab,
        json!(Constraint::required(Resource::NetworkAccess(
            NetReq::Subnet("10.0.0.0/8".into())
        )))
        .to_string(),
        r#"{"optional":false,"resource":{"network-access":{"subnet":"10.0.0.0/8"}}}"#
    );
    test_assert_eq!(
        encode_ip_lowercase,
        json!(Constraint::required(Resource::NetworkBelong(NetReq::IP(
            "172.0.2.81".into()
        ))))
        .to_string(),
        r#"{"optional":false,"resource":{"network-belong":{"ip":"172.0.2.81"}}}"#
    );
    test_assert_eq!(
        encode_untagged_enum,
        json!(Constraint::required(Resource::Gpu(GpuKind::OpenCL))).to_string(),
        r#"{"optional":false,"resource":{"gpu":"open-cl"}}"#
    );
    test_assert_eq!(
        encode_opengl_proper_kebabing,
        json!(Constraint::required(Resource::Gpu(GpuKind::OpenGL))).to_string(),
        r#"{"optional":false,"resource":{"gpu":"open-gl"}}"#
    );
    test_assert_eq!(
        encode_cuda_lowercase,
        json!(Constraint::required(Resource::Gpu(GpuKind::CUDA))).to_string(),
        r#"{"optional":false,"resource":{"gpu":"cuda"}}"#
    );

    fn decode<'s, T: Deserialize<'s>>(json: &'s str) -> T {
        serde_json::from_str(json).unwrap()
    }

    test_assert_eq!(
        decode_absolute_from_number,
        decode::<Constraint>(r#"{"optional":false,"resource":{"memory":200}}"#),
        Constraint::required(Resource::Memory(MemoryReq::Absolute(200)))
    );
    test_assert_eq!(
        decode_minimum_1kb_bound,
        decode::<Constraint>(r#"{"optional":false,"resource":{"memory":"64b"}}"#),
        Constraint::required(Resource::Memory(MemoryReq::Absolute(1)))
    );
    test_assert_eq!(
        decode_upscale_bytes_to_kb,
        decode::<Constraint>(r#"{"optional":false,"resource":{"memory":"157575b"}}"#),
        Constraint::required(Resource::Memory(MemoryReq::Absolute(153)))
    );
    test_assert_eq!(
        decode_downscale_mb,
        decode::<Constraint>(r#"{"optional":false,"resource":{"memory":"35m"}}"#),
        Constraint::required(Resource::Memory(MemoryReq::Absolute(35840)))
    );
    test_assert_eq!(
        decode_string_to_percent,
        decode::<Constraint>(r#"{"optional":false,"resource":{"memory":"2%"}}"#),
        Constraint::required(Resource::Memory(MemoryReq::Percentage(2)))
    );
    test_assert_eq!(
        decode_kebab,
        decode::<Constraint>(
            r#"{"optional":true,"resource":{"network-access":{"subnet":"10.0.100.0/24"}}}"#
        ),
        Constraint::optional(Resource::NetworkAccess(NetReq::Subnet(
            "10.0.100.0/24".into()
        )))
    );
    test_assert_eq!(
        decode_untagged_enum,
        decode::<Constraint>(r#"{"optional":true,"resource":{"gpu":"open-cl"}}"#),
        Constraint::optional(Resource::Gpu(GpuKind::OpenCL))
    );
}
