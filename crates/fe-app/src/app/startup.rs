//! Command-line options: picture files to open, `--bench` (speed test) and `--wgpu` (use the wgpu renderer
//! instead of the default OpenGL one, to test the fallback).

use std::path::PathBuf;

/// What the program was started with.
#[derive(Clone, Debug, Default)]
pub struct StartupOptions {
    pub files: Vec<PathBuf>,
    pub bench: bool,
    pub wgpu: bool,
}

impl StartupOptions {
    /// Reads the command line (the first item, the program itself, is skipped).
    pub fn from_args(args: impl Iterator<Item = String>) -> Self {
        let mut options = Self::default();
        for arg in args.skip(1) {
            match arg.as_str() {
                "--bench" => options.bench = true,
                "--wgpu" => options.wgpu = true,
                "--opengl" => {} // the default now; still accepted
                other => options.files.push(PathBuf::from(other)),
            }
        }
        options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_files_and_switches() {
        let args = ["flash-editor.exe", "a.png", "--bench", "b.jpg", "--wgpu"].map(String::from);
        let o = StartupOptions::from_args(args.into_iter());
        assert!(o.bench && o.wgpu);
        assert_eq!(vec![PathBuf::from("a.png"), PathBuf::from("b.jpg")], o.files);
    }

    #[test]
    fn opengl_is_the_default() {
        let o = StartupOptions::from_args(["flash-editor.exe", "--opengl"].map(String::from).into_iter());
        assert!(!o.wgpu);
        assert!(o.files.is_empty());
    }
}
