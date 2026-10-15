//! P7 neutrino heating (m0_contrat.md §1.3.2, §1.3.4, §2.5.4, §2.9; M0-T54), on shaders/nuheat.wgsl appended to shaders/fft.wgsl:
//! M0-T53's twin (sr_physics::reference::nuheat) in f32 — D(x) = Σ(x) · (S_ν ∗ K_ν)(x) over the active box, K_ν(r) = 1/(r² + 1),
//! D = 0 where X_n ≥ 0.5, and q(x) = f_dep · ΣS_ν · D(x) / ΣD added to E over Δt_n and booked `neutrino_deposited`.
//!
//! The convolution is P2's machinery on K_ν: the kernel's transform for every padded size the world's boxes can take, as its real
//! quarter in f32 ([`super::grav::spectra_of`], the box's size lives only on the GPU, §1.3.3), the rows from fft.wgsl's `conv_rows`
//! and `conv_rows_inverse`, the columns `nu_columns`. S_ν is P6's plane ([`SideFields::s_nu`]).
//!
//! Zero dispatches when ΣS_ν = 0: `nu_tiles` sums S_ν over the box's tiles, `nu_gate` (one workgroup) adds the tiles in a fixed
//! order and writes the sizes of everything after it — all 0 when nothing is emitted — into P7's own buffer, and the dispatches
//! after it are [`Size::Chained`] on it; `nu_close` zeroes the sizes at the pass's end. A frame's latch reaches the chain through
//! the two fixed-size dispatches that open and close it, as P5's rounds: a stopped step runs neither, and the sizes stay 0
//! (latch.rs).

use sr_physics::reference::nuheat::kernel;
use sr_physics::registry::Physics;

use super::boxfit::{BoxFit, ARGS_TILES};
use super::fft::{self, padded, FftError};
use super::grav::spectra_of;
use super::{compute_pipeline, init_buffer, Dispatch, Pass, Size, Step};
use crate::state::{read_buffers, Booking, SideFields, State, BOOK_GROUPS, TERMS};

/// What P7 needs beyond the state: the side fields whose S_ν it reads (P6's, or a test's).
#[derive(Clone, Copy)]
pub struct NuHeatSetup<'a> {
    pub side: &'a SideFields,
}

/// The header of nuheat.wgsl's `NuState`, in words, before the tiles' sums.
const HEADER_WORDS: usize = 8;
/// Byte offsets in the sizes buffer: rows, columns, tiles, one — 3 words each.
const ROWS: u64 = 0;
const COLUMNS: u64 = 12;
const TILES: u64 = 24;
const ONE: u64 = 36;

/// P7's dispatches and its state buffer.
pub struct NuHeat {
    pub(super) dispatches: Vec<Dispatch>,
    state: wgpu::Buffer,
}

/// P7's counters: what the last `nu_gate` found and whether the chain after it ran.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NuHeatReport {
    /// Steps the gate has run on, since the step was made.
    pub steps: u32,
    /// The last gate opened the chain (ΣS_ν > 0).
    pub opened: bool,
    /// The last gate's step ran `nu_share`, the chain's one-workgroup dispatch: what the GPU did with the sizes.
    pub ran: bool,
    /// Σ S_ν and Σ D of the last step, and the scale f_dep ΣS_ν / ΣD (0 when ΣD = 0) — a chain that did not run leaves the last two as
    /// they were.
    pub emitted: f32,
    pub total: f32,
    pub scale: f32,
}

impl NuHeat {
    /// P7 for `state`'s world over `boxfit`'s box, reading Δt_n from `dt_record` and S_ν from `setup.side`, booking into `booking`
    /// `neutrino_deposited`, f_dep from `physics`. Refused when a world axis pads past the FFT's largest length (over 1024 cells).
    pub fn new(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        physics: &Physics,
        boxfit: &BoxFit,
        dt_record: &wgpu::Buffer,
        setup: &NuHeatSetup<'_>,
    ) -> Result<NuHeat, FftError> {
        let world = state.world();
        let (table, offsets, ny_sizes) = spectra_of(world.width, world.height, &kernel)?;
        let max_nx = padded(world.width)?;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nuheat.wgsl"),
            source: wgpu::ShaderSource::Wgsl(
                format!("{}\n{}", include_str!("../../shaders/fft.wgsl"), include_str!("../../shaders/nuheat.wgsl")).into(),
            ),
        });
        let storage = wgpu::BufferUsages::STORAGE;
        let cells = world.cells() as u32;
        let tiles = world.width.div_ceil(8) * world.height.div_ceil(8);
        let f_dep = physics.get("f_dep").expect("physics.json key f_dep") as f32;
        let fft_params = init_buffer(device, "nuheat.fft_params", wgpu::BufferUsages::UNIFORM, &[world.width, 0, 0, 0]);
        let params = init_buffer(device, "nuheat.params", wgpu::BufferUsages::UNIFORM, &[f_dep.to_bits(), ny_sizes, cells, tiles]);
        let twiddles = fft::twiddles(device);
        let kquarter = init_buffer(device, "nuheat.kernel_quarters", storage, &table.iter().map(|v| v.to_bits()).collect::<Vec<_>>());
        let koffsets = init_buffer(device, "nuheat.kernel_offsets", storage, &offsets);
        let work = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nuheat.work"),
            size: 8 * u64::from(max_nx) * u64::from(world.height),
            usage: storage,
            mapped_at_creation: false,
        });
        // The convolution's output: reach, then D in its place; one plane of the world.
        let reach = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nuheat.reach"),
            size: u64::from(cells) * 4,
            usage: storage,
            mapped_at_creation: false,
        });
        let state_buffer = init_buffer(
            device,
            "nuheat.state",
            storage | wgpu::BufferUsages::COPY_SRC,
            &vec![0; HEADER_WORDS + 2 * tiles as usize],
        );
        let sizes = init_buffer(device, "nuheat.sizes", storage | wgpu::BufferUsages::INDIRECT, &[0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1]);
        let book = BOOK_GROUPS.iter().position(|g| TERMS[g.terms.start].writer == Pass::NeutrinoHeating).expect("a booking buffer written by P7");

        // Every entry point binds the subset of group 0 it uses (auto layouts); `entry` maps a binding to its buffer.
        let resource = |binding: u32| -> &wgpu::Buffer {
            match binding {
                0 => boxfit.active(),
                1 => &fft_params,
                2 => &twiddles,
                3 => &work,
                4 => setup.side.s_nu(),
                5 => &reach,
                7 => &params,
                8 => &kquarter,
                9 => &koffsets,
                10 => &state_buffer,
                11 => state.group(0),
                12 => state.group(2),
                13 => booking.group(book),
                14 => dt_record,
                _ => &sizes,
            }
        };
        let dispatch = |label: &'static str, bindings: &[u32], size: Size| {
            let pipeline = compute_pipeline(device, &module, label);
            let entries: Vec<wgpu::BindGroupEntry> =
                bindings.iter().map(|&b| wgpu::BindGroupEntry { binding: b, resource: resource(b).as_entire_binding() }).collect();
            let bind_groups = vec![device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &entries,
            })];
            Dispatch { label, pipeline, bind_groups, size }
        };
        let chained = |offset| Size::Chained(sizes.clone(), offset);
        let one = || Size::Fixed([1, 1, 1]);
        let dispatches = vec![
            dispatch("nu_tiles", &[0, 1, 4, 10], Size::Indirect(boxfit.args().clone(), ARGS_TILES)),
            dispatch("nu_gate", &[0, 10, 15], one()),
            dispatch("conv_rows", &[0, 1, 2, 3, 4], chained(ROWS)),
            dispatch("nu_columns", &[0, 2, 3, 7, 8, 9], chained(COLUMNS)),
            dispatch("conv_rows_inverse", &[0, 1, 2, 3, 5], chained(ROWS)),
            dispatch("nu_deposit", &[0, 1, 5, 7, 10, 11, 12], chained(TILES)),
            dispatch("nu_share", &[0, 7, 10], chained(ONE)),
            dispatch("nu_apply", &[0, 1, 5, 7, 10, 11, 13, 14], chained(TILES)),
            dispatch("nu_close", &[15], one()),
        ];
        Ok(NuHeat { dispatches, state: state_buffer })
    }
}

impl Step {
    /// Whether the step runs P7.
    pub fn runs_neutrino_heating(&self) -> bool {
        self.nuheat.is_some()
    }

    /// P7's counters after every submitted step; none when the step runs no P7. Blocks on the device (tests and headless).
    pub fn read_nuheat(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<NuHeatReport> {
        let p7 = self.nuheat.as_ref()?;
        let v = read_buffers(device, queue, "nuheat.read", std::iter::once(&p7.state));
        let word = |i: usize| v[i].to_bits();
        Some(NuHeatReport {
            steps: word(0),
            opened: word(1) != 0,
            ran: word(0) != 0 && word(2) == word(0),
            emitted: v[4],
            total: v[5],
            scale: v[6],
        })
    }
}
