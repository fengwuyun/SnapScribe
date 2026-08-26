use std::path::{Path, PathBuf};

/// All external executables and models the pipeline needs.
#[derive(Debug, Clone)]
pub struct RuntimePaths {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
    pub asr_exe: PathBuf,
    pub asr_model: PathBuf,
    pub vad_model: PathBuf,
}

impl RuntimePaths {
    /// Locate runtimes for the current execution mode.
    ///
    /// Order per DEVELOPMENT_SPEC §9: packaged resources next to the running
    /// executable first, then development fallbacks inside the repository
    /// (`models/`, `funasr-llamacpp-windows-x64/`, PATH for FFmpeg).
    /// Missing pieces fail loud here so the caller can report one clear message.
    pub fn resolve(resource_dir: Option<&Path>) -> Result<Self, String> {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(res) = resource_dir {
            candidates.push(res.to_path_buf());
        }
        if let Some(dir) = exe_dir {
            // NSIS installs keep bundled resources beside the executable.
            candidates.push(dir);
        }

        let mut missing: Vec<String> = Vec::new();

        let ffmpeg = find_in(&candidates, &["ffmpeg.exe"])
            .or_else(find_ffmpeg_on_path)
            .ok_or_else(|| "未找到 ffmpeg（已尝试打包资源与系统 PATH）".to_string())?;
        let ffprobe = ffmpeg.with_file_name("ffprobe.exe");
        if !ffprobe.is_file() {
            missing.push(format!("ffprobe（期望位于 {}）", ffprobe.display()));
        }

        let repo_root = option_env!("CARGO_MANIFEST_DIR")
            .map(|m| Path::new(m).parent().expect("manifest parent").to_path_buf());

        // Development fallback: bundled binaries live in src-tauri/resources/bin
        // even before packaging puts them beside the executable.
        let mut bin_candidates = candidates.clone();
        if let Some(root) = &repo_root {
            bin_candidates.push(root.join("src-tauri").join("resources").join("bin"));
            bin_candidates.push(root.join("funasr-llamacpp-windows-x64"));
        }

        let mut missing: Vec<String> = Vec::new();

        let ffmpeg = find_in(&bin_candidates, &["ffmpeg.exe"])
            .or_else(find_ffmpeg_on_path)
            .ok_or_else(|| "未找到 ffmpeg（已尝试打包资源与系统 PATH）".to_string())?;
        let ffprobe = ffmpeg.with_file_name("ffprobe.exe");
        if !ffprobe.is_file() {
            missing.push(format!("ffprobe（期望位于 {}）", ffprobe.display()));
        }

        let asr_exe = find_in(&bin_candidates, &["llama-funasr-sensevoice.exe"]);
        let asr_exe = match asr_exe {
            Some(p) => p,
            None => {
                missing.push("llama-funasr-sensevoice.exe".to_string());
                PathBuf::new()
            }
        };

        let model_dir_candidates: Vec<PathBuf> = candidates
            .iter()
            .map(|c| c.join("models"))
            .chain(repo_root.as_ref().map(|r| r.join("models")))
            .collect();
        let asr_model =
            find_in(&model_dir_candidates, &["sensevoice-small-q8.gguf"]).unwrap_or_default();
        if !asr_model.is_file() {
            missing.push("sensevoice-small-q8.gguf".to_string());
        }
        let vad_model = find_in(&model_dir_candidates, &["fsmn-vad.gguf"]).unwrap_or_default();
        if !vad_model.is_file() {
            missing.push("fsmn-vad.gguf".to_string());
        }

        if !missing.is_empty() {
            return Err(format!("运行时资源缺失：{}", missing.join("；")));
        }

        Ok(Self {
            ffmpeg,
            ffprobe,
            asr_exe,
            asr_model,
            vad_model,
        })
    }
}

/// Return the first existing `dir/<name>` across candidate directories.
fn find_in(dirs: &[PathBuf], names: &[&str]) -> Option<PathBuf> {
    dirs.iter().find_map(|d| first_existing(d, names))
}

fn first_existing(dir: &Path, names: &[&str]) -> Option<PathBuf> {
    names.iter().find_map(|n| {
        let p = dir.join(n);
        p.is_file().then_some(p)
    })
}

/// Search PATH for ffmpeg.exe (development machines usually have it installed).
fn find_ffmpeg_on_path() -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|dir| dir.join("ffmpeg.exe"))
        .find(|p| p.is_file())
}
