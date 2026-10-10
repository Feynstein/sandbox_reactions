//! The one GPU test binary (R11): every GPU scope adds a module here and runs `--test gpu <module>::`.
//! Every test that touches a GPU prints the adapter it ran on as `SR-ADAPTER …` (R13); read it with `--nocapture`.

use sr_engine::gpu::Gpu;

/// The device helper every GPU module uses: the adapter named by SR_TEST_ADAPTER (a name substring, as
/// `--adapter`), else wgpu's high-performance pick — which on a three-GPU box may be the 5090 or the Quadro
/// (Hazards), so a test that needs a particular adapter names it. Prints `SR-ADAPTER …`.
#[allow(dead_code)]
pub fn device() -> Gpu {
    let query = std::env::var("SR_TEST_ADAPTER").ok().filter(|s| !s.trim().is_empty());
    let gpu = pollster::block_on(Gpu::new_headless(query.as_deref()))
        .unwrap_or_else(|e| panic!("no test device (SR_TEST_ADAPTER={query:?}): {e}"));
    println!("{}", gpu.info.sr_adapter_line());
    gpu
}

/// M0-T3 — the cell state and the step loop (contract §2.2, §2.8, §1.3.2, §1.3.4).
mod state;

/// M0-T18 — the side fields and the booking layout (contract §2.2, §1.9.1, §2.9, §1.3.4).
mod booking;

/// M0-T19 — P8 complete and the equation of state on the GPU (contract §1.3.2, §2.3.2–§2.3.3, §2.7, §2.9).
mod floors;

/// M0-T2 — the adapter choice (contract §6.1–§6.3, §6.2.2).
mod adapter {
    use super::device;
    use sr_engine::gpu::{match_adapter, AdapterDesc, Gpu, GpuError};

    fn desc(name: &str, backend: wgpu::Backend) -> AdapterDesc {
        AdapterDesc {
            name: name.to_string(),
            backend,
            driver: "test-driver".to_string(),
            device_type: wgpu::DeviceType::DiscreteGpu,
        }
    }

    /// linux-pc's three GPUs as wgpu lists them, the RTX 5090 on two backends (DX12 first, as a Windows box may).
    fn fixed_list() -> Vec<AdapterDesc> {
        vec![
            desc("NVIDIA GeForce RTX 5090", wgpu::Backend::Dx12),
            desc("NVIDIA GeForce RTX 5090", wgpu::Backend::Vulkan),
            desc("Quadro RTX 4000", wgpu::Backend::Vulkan),
            desc("llvmpipe (LLVM 19.1.1, 256 bits)", wgpu::Backend::Vulkan),
        ]
    }

    #[test]
    fn match_is_case_insensitive() {
        let list = fixed_list();
        assert_eq!(match_adapter(&list, "quadro rtx 4000").unwrap(), 2);
        assert_eq!(match_adapter(&list, "QUADRO").unwrap(), 2);
        assert_eq!(match_adapter(&list, "LLVMPIPE").unwrap(), 3);
    }

    #[test]
    fn match_is_a_substring_of_the_name() {
        let list = fixed_list();
        assert_eq!(match_adapter(&list, "4000").unwrap(), 2);
        assert_eq!(match_adapter(&list, "pipe (llvm").unwrap(), 3);
        assert!(match_adapter(&list, "RTX 4000 Quadro").is_err(), "a substring, not a bag of words");
    }

    #[test]
    fn match_prefers_vulkan_when_a_name_matches_on_two_backends() {
        let list = fixed_list();
        assert_eq!(match_adapter(&list, "RTX 5090").unwrap(), 1, "the Vulkan entry, not the DX12 one listed first");
        let only_dx12 = vec![desc("Intel(R) Arc(TM) Graphics", wgpu::Backend::Dx12)];
        assert_eq!(match_adapter(&only_dx12, "arc").unwrap(), 0, "no Vulkan twin: the other backend still matches");
    }

    #[test]
    fn match_on_several_names_takes_vulkan_then_list_order() {
        let list = fixed_list();
        // "RTX" is in the 5090 (two backends) and the Quadro: Vulkan first, then the order wgpu listed them.
        assert_eq!(match_adapter(&list, "rtx").unwrap(), 1);
    }

    #[test]
    fn no_match_is_an_error_naming_every_adapter_seen() {
        let list = fixed_list();
        match match_adapter(&list, "Radeon") {
            Err(GpuError::NoMatch { query, seen }) => {
                assert_eq!(query, "Radeon");
                assert_eq!(seen, list);
                let text = GpuError::NoMatch { query, seen }.to_string();
                for a in &list {
                    assert!(text.contains(&a.name), "the message must name {:?}: {text}", a.name);
                }
                assert!(text.contains("Radeon"), "the message must echo the query: {text}");
            }
            other => panic!("expected NoMatch, got {other:?}"),
        }
    }

    /// Every adapter wgpu lists on this box, once per name (a name on two backends is reached by name through
    /// Vulkan; the rule's own test covers the other twin).
    fn live_adapters() -> Vec<AdapterDesc> {
        let instance = sr_engine::gpu::headless_instance();
        let mut list: Vec<AdapterDesc> = pollster::block_on(sr_engine::gpu::list_adapters(&instance))
            .iter()
            .map(|a| (&a.get_info()).into())
            .collect();
        for a in &list {
            println!("listed: {a}");
        }
        let mut names = std::collections::HashSet::new();
        list.retain(|a| names.insert(a.name.clone()));
        list
    }

    #[test]
    fn live_this_boxs_adapters_are_listed() {
        let expected: &[&str] =
            if cfg!(windows) { &["RTX 4080 Laptop", "Intel(R) Arc"] } else { &["Quadro RTX 4000", "RTX 5090", "llvmpipe"] };
        let list = live_adapters();
        let missing: Vec<&&str> = expected
            .iter()
            .filter(|e| !list.iter().any(|a| a.name.to_lowercase().contains(&e.to_lowercase())))
            .collect();
        assert!(missing.is_empty(), "adapters missing on this box: {missing:?}; wgpu lists: {list:?}");
    }

    #[test]
    fn live_one_device_per_adapter_found_by_a_differently_cased_name() {
        let list = live_adapters();
        assert!(!list.is_empty(), "no adapter listed at all");
        for a in &list {
            // The query is the name in lower case: the device must still be this adapter's (§6.1).
            let query = a.name.to_lowercase();
            let gpu = pollster::block_on(Gpu::new_headless(Some(&query)))
                .unwrap_or_else(|e| panic!("no device for {:?}: {e}", a.name));
            println!("{}", gpu.info.sr_adapter_line());
            assert_eq!(gpu.info.name, a.name, "asked for {query:?}");
            let on_vulkan = gpu.seen.iter().any(|s| s.name == a.name && s.backend == wgpu::Backend::Vulkan);
            if on_vulkan {
                assert_eq!(gpu.info.backend, wgpu::Backend::Vulkan, "{}: Vulkan wins when listed (§6.1)", a.name);
            }
            assert!(gpu.seen.iter().any(|s| s.name == a.name), "the chosen adapter is in the list seen");
        }
    }

    #[test]
    fn live_device_limits_equal_webgpu_defaults_never_the_adapters_own() {
        for a in live_adapters() {
            let gpu = pollster::block_on(Gpu::new_headless(Some(&a.name))).unwrap();
            println!("{}", gpu.info.sr_adapter_line());
            let adapter_limits = gpu.adapter.limits();
            assert_eq!(gpu.device.limits(), wgpu::Limits::default(), "{}: the device's limits", a.name);
            assert!(gpu.device.features().is_empty(), "{}: no native-only feature (§6.2.2)", a.name);
            // The point of the case: this adapter could have given more, and the device must not have taken it.
            println!(
                "{}: adapter allows {} B workgroup storage, device has {} B",
                a.name,
                adapter_limits.max_compute_workgroup_storage_size,
                gpu.device.limits().max_compute_workgroup_storage_size
            );
            assert_eq!(gpu.device.limits().max_compute_workgroup_storage_size, 16_384);
            assert_eq!(gpu.device.limits().max_compute_invocations_per_workgroup, 256);
            assert_eq!(gpu.device.limits().max_storage_buffers_per_shader_stage, 8);
        }
    }

    #[test]
    fn live_default_pick_prints_which_adapter_it_is() {
        // SR_TEST_ADAPTER unset: wgpu's high-performance pick. No later test may rely on it being the 5090.
        let gpu = pollster::block_on(Gpu::new_headless(None)).expect("a high-performance adapter");
        println!("{}", gpu.info.sr_adapter_line());
        assert!(gpu.seen.iter().any(|s| s == &gpu.info), "the pick is one of the adapters listed");
        // And the helper every later GPU module uses reaches a device too (SR_TEST_ADAPTER, else this pick).
        let _ = device();
    }

    #[test]
    fn live_no_match_names_every_adapter_the_box_has() {
        let listed = live_adapters();
        let err = pollster::block_on(Gpu::new_headless(Some("no-such-gpu-xyz"))).err().expect("must not match");
        let text = err.to_string();
        println!("{text}");
        assert!(matches!(err, GpuError::NoMatch { .. }), "{err:?}");
        for a in &listed {
            assert!(text.contains(&a.name), "the error must name {:?}: {text}", a.name);
        }
    }
}
