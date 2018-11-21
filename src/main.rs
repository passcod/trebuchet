fn main() {
    println!("Hello, world!");
}

struct Worker {
    name: String,
    inputs: Vec<DataDef>,
    outputs: Vec<DataDef>,
    constraints: Vec<Constraint>,
}

struct DataDef {
    name: String,
    optional: bool,
    datatype: DataType,
}

enum DataType {
    Stream,
    Bool,
    Int,
    Float,
    String,
    Binary,
    Json,
}

struct Constraint {
    resource: Resource,
    optional: bool, // will be scheduled on matching nodes preferentially, but can run on non-matching nodes
}

enum Resource {
    Memory(ScalarReq),
    Cpu(ScalarReq),
    Gpu(GpuKind), // not the brand or power, more the tech interface available
    NetworkBelong(NetReq),
    NetworkAccess(NetReq),
}

enum ScalarReq {
    Absolute(usize),
    Percentage(i8),
}

enum GpuKind {
    OpenGL,
    CUDA,
    OpenCL,
}

enum NetReq {
    Subnet(String), // belong: has ip within this subnet; access: can route to this subnet
    IP(String), // belong: has this ip; access: can ping this hostname
    Name(String), // belong: has this hostname; access: can resolve & ping this hostname
}
