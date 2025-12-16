use std::fs;
use std::io::Error as IoError;
use std::time::Instant;

use xshell::{Shell, cmd};

use super::{Metric, MetricRecorder, Result};

/// Measure size of the compiled arm64 libxul.so library file
pub struct AndroidLibrarySize;

impl MetricRecorder for AndroidLibrarySize {
    fn name(&self) -> &'static str {
        "android-library-size"
    }

    fn description(&self) -> &'static str {
        "Android Library Size"
    }

    fn record(&self, sh: &Shell) -> Result<Vec<Metric>> {
        build_android(&sh)?;

        let lib_file = sh
            .current_dir()
            .join("glean-core/android-native/build/rustJniLibs/android/arm64-v8a/libxul.so");
        let Ok(metadata) = lib_file.metadata() else {
            return Err(IoError::other("no metadata").into());
        };
        if !metadata.is_file() {
            return Err(IoError::other("not a file").into());
        }

        let lib_size = metadata.len();
        let metric = Metric {
            name: String::from("lib size arm64-v8"),
            unit: String::from("bytes"),
            value: lib_size,
        };

        Ok(vec![metric])
    }
}

fn build_android(sh: &Shell) -> Result<()> {
    let _env = sh.push_env("CI", "1");
    let _env = sh.push_env("GRADLE_OPTS", "-Dorg.gradle.daemon=false");

    if fs::exists("local.properties").unwrap_or(false) {
        let dest = sh.current_dir().join("local.properties");
        fs::copy("local.properties", dest)?;
    }

    let now = Instant::now();
    cmd!(sh, "./gradlew --no-daemon :glean-native:cargoBuildArm64").run()?;
    let duration = now.elapsed();
    println!("Build took: {:?}", duration);

    Ok(())
}
