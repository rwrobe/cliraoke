use soloud::Backend;

pub(crate) fn get_platform_backend() -> Backend {
    #[cfg(target_os = "windows")]
    {
        Backend::Winmm;
    }
    #[cfg(target_os = "linux")]
    {
        Backend::Alsa;
    }
    #[cfg(target_os = "macos")]
    {
        Backend::CoreAudio
    }
}