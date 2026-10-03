cfg_select! {
    all(target_family = "unix", not(target_os = "espidf")) => {
        mod unix;
        pub use unix::hostname;
    }
    // Windows 8+ uses `GetHostNameW`. The Windows 7 target uses ANSI `gethostname`.
    target_os = "windows" => {
        mod windows;
        pub use windows::hostname;
    }
    _ => {
        mod unsupported;
        pub use unsupported::hostname;
    }
}
