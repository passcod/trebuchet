use crate::proto::*;
use hostname::get_hostname;
use ipnet::IpNet;
use lazy_static::lazy_static;
use log::error;
use std::{fmt, io::Result as IoResult};
use systemstat::Platform;

pub struct System {
    platform: systemstat::System,
    pub detect_gpu: bool,
}

impl System {
    /// Initialises the system constraint checker.
    pub fn new() -> Self {
        Self {
            platform: systemstat::System::new(),
            detect_gpu: false,
        }
    }

    /// Retrieves the system available memory in KiB.
    pub fn available_memory(&self) -> IoResult<usize> {
        self.platform.memory().map(|mem| mem.free.as_usize() / 1024)
    }

    /// Retrieves the system available memory as percentage of the total.
    #[allow(clippy::cast_precision_loss)]
    pub fn available_memory_percent(&self) -> IoResult<f32> {
        self.platform
            .memory()
            .map(|mem| (mem.free.as_usize() as f32) / (mem.total.as_usize() as f32) * 100_f32)
    }

    /// Retrieves the system available load percentage.
    ///
    /// If the `mpstat` tool is available, it uses the idle average. // TODO
    ///
    /// Otherwise, it's calculated as the 1-minute average load divided by the
    /// number of cores, rendered as a percentage subtracted from 100%.
    #[allow(clippy::cast_precision_loss)]
    pub fn available_load(&self) -> IoResult<f32> {
        self.platform
            .load_average()
            .map(|load| 100_f32 - (load.one / num_cpus::get() as f32) * 100_f32)
    }

    /// Retrieves the system hostname.
    pub fn hostname(&self) -> Option<String> {
        get_hostname()
    }

    /// Assumes true since very few platforms won't.
    ///
    /// TODO: Actually check. (How?!)
    pub fn has_opengl(&self) -> bool {
        if !self.detect_gpu {
            return false;
        }

        true
    }

    /// Checks OpenCL availability by listing the system's OpenCL platforms.
    ///
    /// Only checks once per running instance.
    pub fn has_opencl(&self) -> bool {
        if !self.detect_gpu {
            return false;
        }

        lazy_static! {
            static ref OCL_AVAILABLE: Option<bool> = ocl_core::get_platform_ids()
                .ok()
                .map(|list| !list.is_empty());
        }

        OCL_AVAILABLE.unwrap_or(false)
    }

    /// Retrieves all IPs associated to all interfaces of the system.
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
                    }
                    .parse()
                    .expect(&format!("Malformed IP address from system {:?}", addr)),
                );
            }
        }
        Ok(nets)
    }

    /// Checks a single resource constraint against the system.
    pub fn check_resource(&self, res: &Resource) -> IoResult<bool> {
        Ok(match res {
            Resource::Memory(MemoryReq::Absolute(free)) => self.available_memory()? >= *free,
            Resource::Memory(MemoryReq::Percentage(available)) => {
                &self.available_memory_percent()? >= available
            }
            Resource::Cpu(CpuReq::Load(available)) => &self.available_load()? >= available,
            Resource::Gpu(GpuKind::OpenGL) => self.has_opengl(),
            Resource::Gpu(GpuKind::OpenCL) => self.has_opencl(),
            Resource::NetworkBelong(NetReq::IP(ip)) => self.belonging_ips()?.contains(ip),
            Resource::NetworkBelong(NetReq::Name(host)) => {
                self.hostname().map_or(false, |name| host == &name)
            }
            Resource::NetworkBelong(NetReq::Subnet(sub)) => {
                self.belonging_ips()?.iter().any(|ip| sub.contains(ip))
            }
            _ => false,
        })
    }

    /// Returns a pass and a score for the system based on the given constraints.
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

impl fmt::Debug for System {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("System")
            .field("platform", &"systemstat::Platform<...>")
            .field("detect_gpu", &self.detect_gpu)
            .finish()
    }
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}
