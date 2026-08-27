use std::process::Command;

/// Windows allocates a new visible console window when a GUI application
/// spawns a console-subsystem child (ffmpeg, ffprobe, the ASR runtime).
/// Silence it so the app never flashes console windows. No-op on other
/// platforms where this flag does not exist.
pub fn hide_console(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW: the child runs with no console window at all.
        cmd.creation_flags(0x0800_0000);
    }
    #[cfg(not(windows))]
    {
        let _ = cmd;
    }
    cmd
}
