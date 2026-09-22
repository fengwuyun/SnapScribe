use std::path::{Path, PathBuf};

/// All external executables and models the pipeline needs.
#[derive(Debug, Clone)]
pub struct RuntimePaths {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
    pub asr_exe: PathBuf,
    pub asr_model: PathBuf,
    pub vad_model: PathBuf,
    pub diarization: Option<DiarizationRuntime>,
}

#[derive(Debug, Clone)]
pub struct DiarizationRuntime {
    pub exe: PathBuf,
    pub segmentation_model: PathBuf,
    pub embedding_model: PathBuf,
}

impl RuntimePaths {
    /// Locate runtimes for the current execution mode.
    ///
    /// Search order per DEVELOPMENT_SPEC §9:
    /// 1. packaged resource dir and its `resources/` subdirectory (NSIS layout),
    /// 2. directories beside the running executable (``, `resources`, `bin`),
    /// 3. development fallbacks inside the repository
    ///    (`src-tauri/resources/{bin,models}`, `funasr-llamacpp-windows-x64`),
    /// 4. system PATH (ffmpeg only).
    ///
    /// Missing pieces fail loud here so the caller can report one clear message.
    pub fn resolve(resource_dir: Option<&Path>) -> Result<Self, String> {
        let bases = candidate_bases(resource_dir);

        let ffmpeg = find_file(&bases, "ffmpeg.exe")
            .or_else(find_on_path("ffmpeg.exe"))
            .ok_or_else(|| format!("未找到 ffmpeg。已尝试：{}", display_dirs(&bases)))?;
        let ffprobe_dir = ffmpeg.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let ffprobe = ffprobe_dir.join("ffprobe.exe");
        if !ffprobe.is_file() {
            return Err(format!("未找到 ffprobe（期望位于 {}）", ffprobe.display()));
        }

        let asr_exe = find_file(&bases, "llama-funasr-sensevoice.exe").ok_or_else(|| {
            format!(
                "未找到识别程序 llama-funasr-sensevoice.exe。已尝试：{}",
                display_dirs(&bases)
            )
        })?;

        // Models live in a `models` sibling of the binaries; also accept them
        // directly inside any base directory.
        let model_dirs = bases
            .iter()
            .flat_map(|b| [b.join("models"), b.clone()])
            .collect::<Vec<_>>();
        let asr_model = find_file(&model_dirs, "sensevoice-small-q8.gguf").ok_or_else(|| {
            format!(
                "未找到模型 sensevoice-small-q8.gguf。已尝试：{}",
                display_dirs(&model_dirs)
            )
        })?;
        let vad_model = find_file(&model_dirs, "fsmn-vad.gguf").ok_or_else(|| {
            format!(
                "未找到模型 fsmn-vad.gguf。已尝试：{}",
                display_dirs(&model_dirs)
            )
        })?;
        let diarization = find_file(&bases, "sherpa-onnx-offline-speaker-diarization.exe")
            .and_then(|exe| {
                let segmentation_model = find_relative(
                    &model_dirs,
                    Path::new("speaker-segmentation").join("model.int8.onnx"),
                )?;
                let embedding_model = find_relative(
                    &model_dirs,
                    Path::new("speaker-embedding").join("3dspeaker.onnx"),
                )?;
                Some(DiarizationRuntime {
                    exe,
                    segmentation_model,
                    embedding_model,
                })
            });

        Ok(Self {
            ffmpeg,
            ffprobe,
            asr_exe,
            asr_model,
            vad_model,
            diarization,
        })
    }
}

/// Directories that may directly contain the bundled executables.
fn candidate_bases(resource_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut bases: Vec<PathBuf> = Vec::new();
    if let Some(res) = resource_dir {
        bases.push(res.to_path_buf());
        bases.push(res.join("resources"));
        bases.push(res.join("resources").join("bin"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            bases.push(dir.to_path_buf());
            bases.push(dir.join("resources"));
            bases.push(dir.join("resources").join("bin"));
            bases.push(dir.join("bin"));
        }
    }
    if let Some(manifest) = option_env!("CARGO_MANIFEST_DIR") {
        if let Some(root) = Path::new(manifest).parent() {
            bases.push(root.join("src-tauri").join("resources").join("bin"));
            bases.push(root.join("src-tauri").join("resources"));
            bases.push(root.join("funasr-llamacpp-windows-x64"));
        }
    }
    bases.dedup();
    bases
}

/// First existing `dir/name` across candidate directories.
fn find_file(dirs: &[PathBuf], name: &str) -> Option<PathBuf> {
    dirs.iter().find_map(|dir| {
        let candidate = dir.join(name);
        candidate.is_file().then_some(candidate)
    })
}

fn find_relative(dirs: &[PathBuf], relative: PathBuf) -> Option<PathBuf> {
    dirs.iter().find_map(|dir| {
        let candidate = dir.join(&relative);
        candidate.is_file().then_some(candidate)
    })
}

/// Search PATH for an executable (development machines often have ffmpeg installed).
fn find_on_path(name: &str) -> impl Fn() -> Option<PathBuf> + '_ {
    move || {
        let path_var = std::env::var_os("PATH")?;
        std::env::split_paths(&path_var)
            .map(|dir| dir.join(name))
            .find(|p| p.is_file())
    }
}

fn display_dirs(dirs: &[PathBuf]) -> String {
    const MAX_SHOWN: usize = 6;
    let shown: Vec<String> = dirs
        .iter()
        .take(MAX_SHOWN)
        .map(|d| d.to_string_lossy().into_owned())
        .chain((dirs.len() > MAX_SHOWN).then(|| "…".to_string()))
        .collect();
    shown.join("、")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Development-mode self-check: on a prepared checkout this machine must
    /// resolve every runtime piece from the repository fallbacks.
    #[test]
    fn resolves_full_runtime_in_dev_checkout() {
        let runtime = RuntimePaths::resolve(None).expect("resolve should succeed");
        assert!(
            runtime.ffmpeg.is_file(),
            "ffmpeg at {}",
            runtime.ffmpeg.display()
        );
        assert!(runtime.ffprobe.is_file());
        assert!(runtime.asr_exe.is_file());
        assert!(runtime.asr_model.is_file());
        assert!(runtime.vad_model.is_file());
    }

    #[test]
    fn find_file_skips_missing_directories() {
        let dirs = vec![PathBuf::from("Z:/definitely-missing"), std::env::temp_dir()];
        std::fs::write(std::env::temp_dir().join("snapscribe-probe.txt"), b"x").unwrap();
        let found = find_file(&dirs, "snapscribe-probe.txt");
        assert!(found.is_some_and(|p| p.is_file()));
        std::fs::remove_file(std::env::temp_dir().join("snapscribe-probe.txt")).ok();
    }

    /// Simulates the NSIS install layout `<install>/resources/{bin,models}` and
    /// asserts every runtime is found inside that directory tree.
    #[test]
    fn resolves_installed_layout_under_resources() {
        let fake =
            std::env::temp_dir().join(format!("snapscribe-install-test-{}", std::process::id()));
        let bin = fake.join("resources").join("bin");
        let models = fake.join("resources").join("models");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::create_dir_all(&models).unwrap();
        for name in ["ffmpeg.exe", "ffprobe.exe", "llama-funasr-sensevoice.exe"] {
            std::fs::write(bin.join(name), b"x").unwrap();
        }
        for name in ["sensevoice-small-q8.gguf", "fsmn-vad.gguf"] {
            std::fs::write(models.join(name), b"x").unwrap();
        }

        let runtime = RuntimePaths::resolve(Some(&fake)).expect("installed layout should resolve");
        assert!(
            runtime.ffmpeg.starts_with(&fake),
            "ffmpeg must come from the installed layout, got {}",
            runtime.ffmpeg.display()
        );
        assert!(runtime.ffprobe.starts_with(&fake));
        assert!(runtime.asr_exe.starts_with(&fake));
        assert!(runtime.asr_model.starts_with(&fake));
        assert!(runtime.vad_model.starts_with(&fake));
        std::fs::remove_dir_all(&fake).ok();
    }

    #[test]
    fn resolves_optional_diarization_layout() {
        let fake = std::env::temp_dir().join(format!(
            "snapscribe-diarization-install-test-{}",
            uuid::Uuid::new_v4()
        ));
        let bin = fake.join("resources").join("bin");
        let models = fake.join("resources").join("models");
        std::fs::create_dir_all(models.join("speaker-segmentation")).unwrap();
        std::fs::create_dir_all(models.join("speaker-embedding")).unwrap();
        std::fs::create_dir_all(&bin).unwrap();
        for name in ["ffmpeg.exe", "ffprobe.exe", "llama-funasr-sensevoice.exe"] {
            std::fs::write(bin.join(name), b"x").unwrap();
        }
        std::fs::write(bin.join("sherpa-onnx-offline-speaker-diarization.exe"), b"x").unwrap();
        for name in ["sensevoice-small-q8.gguf", "fsmn-vad.gguf"] {
            std::fs::write(models.join(name), b"x").unwrap();
        }
        std::fs::write(
            models.join("speaker-segmentation").join("model.int8.onnx"),
            b"x",
        )
        .unwrap();
        std::fs::write(
            models.join("speaker-embedding").join("3dspeaker.onnx"),
            b"x",
        )
        .unwrap();

        let runtime = RuntimePaths::resolve(Some(&fake)).unwrap();
        let diarization = runtime.diarization.expect("diarization should resolve");

        assert!(diarization.exe.starts_with(&fake));
        assert!(diarization.segmentation_model.starts_with(&fake));
        assert!(diarization.embedding_model.starts_with(&fake));
        std::fs::remove_dir_all(fake).ok();
    }
}
