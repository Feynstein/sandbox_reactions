//! The headless device and the adapter choice (m0_contrat.md §6.1–§6.3, §3.3).
//!
//! `--adapter <substring>` is matched case-insensitively against the adapter's name; where a name matches on two
//! backends Vulkan wins (§6.1), then the order wgpu listed them in. No match is an error naming every adapter seen
//! (the binary's exit 3). No `--adapter` is wgpu's high-performance preference. The device is created with
//! `Limits::default()` and no feature, so what runs here runs in the browser (§6.2.2).

use std::fmt;

/// What the choice reads of an adapter — plain data, so the matching rule runs on a fixed list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterDesc {
    pub name: String,
    pub backend: wgpu::Backend,
    pub driver: String,
    pub device_type: wgpu::DeviceType,
}

impl From<&wgpu::AdapterInfo> for AdapterDesc {
    fn from(info: &wgpu::AdapterInfo) -> Self {
        Self {
            name: info.name.clone(),
            backend: info.backend,
            driver: info.driver.clone(),
            device_type: info.device_type,
        }
    }
}

impl fmt::Display for AdapterDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({:?}, {:?}, driver {})", self.name, self.backend, self.device_type, self.driver)
    }
}

impl AdapterDesc {
    /// The first stdout line of every GPU run (§3.3): `SR-ADAPTER name=… backend=… driver=…`.
    pub fn sr_adapter_line(&self) -> String {
        format!("SR-ADAPTER name={} backend={:?} driver={}", self.name, self.backend, self.driver)
    }
}

#[derive(Debug)]
pub enum GpuError {
    /// `--adapter <query>` matched no adapter's name; `seen` is every adapter wgpu listed.
    NoMatch { query: String, seen: Vec<AdapterDesc> },
    /// No adapter at all (or none the high-performance request could pick).
    NoAdapter { seen: Vec<AdapterDesc>, cause: String },
    /// The adapter cannot give WebGPU's default limits (§6.2.2); `limits` names the ones short.
    LimitsBelowDefault { adapter: AdapterDesc, limits: Vec<String> },
    /// wgpu refused the device for another reason.
    Device { adapter: AdapterDesc, cause: String },
}

fn seen_list(seen: &[AdapterDesc]) -> String {
    if seen.is_empty() {
        return "none".to_string();
    }
    seen.iter().enumerate().map(|(i, a)| format!("[{i}] {a}")).collect::<Vec<_>>().join("; ")
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuError::NoMatch { query, seen } => {
                write!(f, "no adapter name contains {query:?}; adapters seen: {}", seen_list(seen))
            }
            GpuError::NoAdapter { seen, cause } => {
                write!(f, "no GPU adapter ({cause}); adapters seen: {}", seen_list(seen))
            }
            GpuError::LimitsBelowDefault { adapter, limits } => write!(
                f,
                "adapter {adapter} is below WebGPU's default limits: {}",
                limits.join(", ")
            ),
            GpuError::Device { adapter, cause } => write!(f, "adapter {adapter} refused the device: {cause}"),
        }
    }
}

impl std::error::Error for GpuError {}

/// Lower is better. Vulkan first (§6.1); the rest keep wgpu's order through the stable sort.
fn backend_rank(backend: wgpu::Backend) -> u8 {
    match backend {
        wgpu::Backend::Vulkan => 0,
        _ => 1,
    }
}

/// The matching rule: the index in `list` of the adapter `query` names — a case-insensitive substring of the name,
/// Vulkan over any other backend, then list order — or `NoMatch` carrying the whole list.
pub fn match_adapter(list: &[AdapterDesc], query: &str) -> Result<usize, GpuError> {
    let needle = query.to_lowercase();
    list.iter()
        .enumerate()
        .filter(|(_, a)| a.name.to_lowercase().contains(&needle))
        .min_by_key(|(i, a)| (backend_rank(a.backend), *i))
        .map(|(i, _)| i)
        .ok_or_else(|| GpuError::NoMatch { query: query.to_string(), seen: list.to_vec() })
}

/// Every adapter wgpu lists on the instance's backends, in wgpu's order (the no-op backend left out).
pub async fn list_adapters(instance: &wgpu::Instance) -> Vec<wgpu::Adapter> {
    instance
        .enumerate_adapters(wgpu::Backends::all())
        .await
        .into_iter()
        .filter(|a| a.get_info().backend != wgpu::Backend::Noop)
        .collect()
}

/// A headless device (no surface, §6.3) and the adapter it came from.
pub struct Gpu {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub info: AdapterDesc,
    /// Every adapter the instance listed, the chosen one included.
    pub seen: Vec<AdapterDesc>,
}

impl Gpu {
    /// `query` is `--adapter`'s substring; `None` is wgpu's high-performance preference.
    pub async fn new_headless(query: Option<&str>) -> Result<Gpu, GpuError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapters = list_adapters(&instance).await;
        let seen: Vec<AdapterDesc> = adapters.iter().map(|a| (&a.get_info()).into()).collect();
        let adapter = match query {
            Some(q) => adapters.into_iter().nth(match_adapter(&seen, q)?).expect("index from the same list"),
            None => instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    ..Default::default()
                })
                .await
                .map_err(|e| GpuError::NoAdapter { seen: seen.clone(), cause: e.to_string() })?,
        };
        let info: AdapterDesc = (&adapter.get_info()).into();

        // WebGPU's defaults, never the adapter's own limits: a shader that passes here passes in the browser (§6.2.2).
        let wanted = wgpu::Limits::default();
        let mut short = Vec::new();
        let offered = adapter.limits();
        wanted.check_limits_with_fail_fn(&offered, false, |name, need, have| {
            short.push(format!("{name}: WebGPU default {need}, adapter {have}"));
        });
        if !short.is_empty() {
            return Err(GpuError::LimitsBelowDefault { adapter: info, limits: short });
        }
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("sr-engine headless"),
                required_features: wgpu::Features::empty(),
                required_limits: wanted,
                ..Default::default()
            })
            .await
            .map_err(|e| GpuError::Device { adapter: info.clone(), cause: e.to_string() })?;
        Ok(Gpu { instance, adapter, device, queue, info, seen })
    }
}
