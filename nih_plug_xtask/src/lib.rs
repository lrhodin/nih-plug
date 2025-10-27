use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

mod symbols;
mod util;

/// Re-export for the main function.
pub use anyhow::Result;

fn build_usage_string(command_name: &str) -> String {
    format!(
        "Usage:
  {command_name} bundle <package> [--release]
  {command_name} bundle -p <package1> -p <package2> ... [--release]

  {command_name} bundle-universal <package> [--release]  (macOS only)
  {command_name} bundle-universal -p <package1> -p <package2> ... [--release]  (macOS only)

  {command_name} xcode-build <package> [--release]  (macOS only)
  {command_name} xcode-build -p <package1> -p <package2> ... [--release]  (macOS only)

  All other 'cargo build' options are supported, including '--target' and '--profile'."
    )
}

/// Any additional configuration that might be useful for creating plugin bundles, stored as
/// `bundler.toml` alongside the workspace's main `Cargo.toml` file.
type BundlerConfig = HashMap<String, PackageConfig>;

#[derive(Debug, Clone, Deserialize)]
struct PackageConfig {
    name: Option<String>,
}

/// The target we're generating a plugin for. This can be either the native target or a cross
/// compilation target, so to reduce redundancy when determining the correct bundle paths we'll use
/// an enum for this.
#[derive(Debug, Clone, Copy)]
pub enum CompilationTarget {
    Linux(Architecture),
    MacOS(Architecture),
    /// A special case for lipo'd `x86_64-apple-darwin` and `aarch64-apple-darwin` builds.
    MacOSUniversal,
    Windows(Architecture),
}

#[derive(Debug, Clone, Copy)]
pub enum Architecture {
    X86,
    X86_64,
    RISCV64,
    // There are also a ton of different 32-bit ARM architectures, we'll just pretend they don't
    // exist for now
    AArch64,
}

/// The type of a MacOS bundle.
#[derive(Debug, Clone, Copy)]
pub enum BundleType {
    Plugin,
    Binary,
}

/// The main xtask entry point function. See the readme for instructions on how to use this.
pub fn main() -> Result<()> {
    let args = std::env::args().skip(1);
    main_with_args("cargo xtask", args)
}

/// The main xtask entry point function, but with custom command line arguments. `args` should not
/// contain the command name, so you should always skip at least one argument from
/// `std::env::args()` before passing it to this function.
pub fn main_with_args(command_name: &str, args: impl IntoIterator<Item = String>) -> Result<()> {
    chdir_workspace_root()?;
    let cargo_metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .exec()
        .context("Could not parse `cargo-metadata`")?;
    let target_dir = cargo_metadata.target_directory.as_std_path();

    let mut args = args.into_iter();
    let usage_string = build_usage_string(command_name);
    let command = args
        .next()
        .with_context(|| format!("Missing command name\n\n{usage_string}",))?;
    match command.as_str() {
        "bundle" => {
            // For convenience's sake we'll allow building multiple packages with `-p` just like
            // cargo build, but you can also build a single package without specifying `-p`. Since
            // multiple packages can be built in parallel if we pass all of these flags to a single
            // `cargo build` we'll first build all of these packages and only then bundle them.
            let (packages, other_args) = split_bundle_args(args, &usage_string)?;

            // As explained above, for efficiency's sake this is a two step process
            build(&packages, &other_args)?;

            bundle(target_dir, &packages[0], &other_args, false)?;
            for package in packages.into_iter().skip(1) {
                bundle(target_dir, &package, &other_args, false)?;
            }

            Ok(())
        }
        "bundle-universal" => {
            // The same as `--bundle`, but builds universal binaries for macOS Cargo will also error
            // out on duplicate `--target` options, but it seems like a good idea to preemptively
            // abort the bundling process if that happens
            let (packages, other_args) = split_bundle_args(args, &usage_string)?;

            for arg in &other_args {
                if arg == "--target" || arg.starts_with("--target=") {
                    anyhow::bail!(
                        "'{command_name} xtask bundle-universal' is incompatible with the '{arg}' \
                         option."
                    )
                }
            }

            // We can just use the regular build function here. There's sadly no way to build both
            // targets in parallel, so this will likely take twice as logn as a regular build.
            // TODO: Explicitly specifying the target even on the native target causes a rebuild in
            //       the target `target/<target_triple>` directory. This makes bundling much simpler
            //       because there's no conditional logic required based on the current platform,
            //       but it does waste some resources and requires a rebuild if the native target
            //       was already built.
            let mut x86_64_args = other_args.clone();
            x86_64_args.push(String::from("--target=x86_64-apple-darwin"));
            build(&packages, &x86_64_args)?;
            let mut aarch64_args = other_args.clone();
            aarch64_args.push(String::from("--target=aarch64-apple-darwin"));
            build(&packages, &aarch64_args)?;

            // This `true` indicates a universal build. This will cause the two sets of built
            // binaries to beq lipo'd together into universal binaries before bundling
            bundle(target_dir, &packages[0], &other_args, true)?;
            for package in packages.into_iter().skip(1) {
                bundle(target_dir, &package, &other_args, true)?;
            }

            Ok(())
        }
        "xcode-build" => {
            // Build AUv3 plugins using Xcode. This command:
            // 1. Builds the Rust library with AUv3 features
            // 2. Runs the build_rust.sh script to prepare the Swift project
            // 3. Uses xcodebuild to build the Swift app extension
            // 4. Integrates with the existing bundling system
            let (packages, other_args) = split_bundle_args(args, &usage_string)?;

            // Check if we're on macOS
            if !cfg!(target_os = "macos") {
                anyhow::bail!("xcode-build is only supported on macOS");
            }

            // For now, we'll build the main nih-plug library with AUv3 features
            // The AUv3 FFI layer includes a test plugin for demonstration
            eprintln!("Building NIH-plug with AUv3 features...");
            let mut auv3_args = other_args.clone();
            auv3_args.push(String::from("--features"));
            auv3_args.push(String::from("auv3"));
            
            // Build the main library with AUv3 features
            let status = Command::new("cargo")
                .arg("build")
                .args(&auv3_args)
                .status()
                .context("Could not build NIH-plug with AUv3 features")?;

            if !status.success() {
                anyhow::bail!("Failed to build NIH-plug with AUv3 features");
            }

            // Build the Swift app extension using Xcode
            xcode_build_auv3("nih-plug")?;

            // Also create the regular bundle for consistency
            bundle(target_dir, &packages[0], &other_args, false)?;
            for package in packages.into_iter().skip(1) {
                bundle(target_dir, &package, &other_args, false)?;
            }

            Ok(())
        }
        // This is only meant to be used by the CI, since using awk for this can be a bit spotty on
        // macOS
        "known-packages" => list_known_packages(),
        _ => anyhow::bail!("Unknown command '{command}'\n\n{usage_string}"),
    }
}

/// Change the current directory into the Cargo workspace's root.
///
/// This is using a heuristic to find the workspace root. It considers all ancestor directories of
/// either `CARGO_MANIFEST_DIR` or the current directory, and finds the leftmost one containing a
/// `Cargo.toml` file.
pub fn chdir_workspace_root() -> Result<()> {
    // This is either the directory of the xtask binary when using `nih_plug_xtask` normally, or any
    // random project when using it through `cargo nih-plug`.
    let project_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|_| std::env::current_dir())
        .context(
            "'$CARGO_MANIFEST_DIR' was not set and the current working directory could not be \
             found",
        )?;

    let workspace_root = project_dir
        .ancestors()
        .filter(|dir| dir.join("Cargo.toml").exists())
        // The ancestors are ordered starting from `project_dir` going up to the filesystem root. So
        // this is the leftmost matching ancestor.
        .last()
        .with_context(|| {
            format!(
                "Could not find a 'Cargo.toml' file in '{}' or any of its parent directories",
                project_dir.display()
            )
        })?;

    std::env::set_current_dir(workspace_root)
        .context("Could not change to workspace root directory")
}

/// Build one or more packages using the provided `cargo build` arguments. This should be called
/// before calling [`bundle()`]. This requires the current working directory to have been set to
/// the workspace's root using [`chdir_workspace_root()`].
pub fn build(packages: &[String], args: &[String]) -> Result<()> {
    let package_args = packages.iter().flat_map(|package| ["-p", package]);

    // Check if we need to build with auv3 features for AUv3 bundling
    let mut build_args: Vec<String> = Vec::new();
    build_args.extend(package_args.map(|s| s.to_string()));
    build_args.extend(args.iter().cloned());
    
    // If we're building nih_plug and it's for AUv3 bundling, add the auv3 feature
    if packages.contains(&"nih_plug".to_string()) {
        build_args.push("--features".to_string());
        build_args.push("auv3".to_string());
    }

    let status = Command::new("cargo")
        .arg("build")
        .args(&build_args)
        .status()
        .with_context(|| format!("Could not call cargo to build {}", packages.join(", ")))?;
    if !status.success() {
        anyhow::bail!("Could not build {}", packages.join(", "));
    } else {
        Ok(())
    }
}

/// Bundle a package that was previously built by a call to [`build()`] using the provided `cargo
/// build` arguments. These two functions are split up because building can be done in parallel by
/// Cargo itself while bundling is sequential. Options from the `bundler.toml` file in the
/// workspace's root are respected (see
/// <https://github.com/robbert-vdh/nih-plug/blob/master/bundler.toml>). This requires the current
/// working directory to have been set to the workspace's root using [`chdir_workspace_root()`].
///
/// If the package also exposes a binary target in addition to a library (or just a binary, in case
/// the binary target has a different name) then this will also be copied into the `bundled`
/// directory.
///
/// Normally this respects the `--target` option for cross compilation. If the `universal` option is
/// specified instead, then this will assume both `x86_64-apple-darwin` and `aarch64-apple-darwin`
/// have been built and it will try to lipo those together instead.
pub fn bundle(target_dir: &Path, package: &str, args: &[String], universal: bool) -> Result<()> {
    let mut build_type_dir = "debug";
    let mut cross_compile_target: Option<String> = None;
    for arg_idx in (0..args.len()).rev() {
        let arg = &args[arg_idx];
        match arg.as_str() {
            "--profile" => {
                // Since Rust 1.57 you can have custom profiles
                build_type_dir = args.get(arg_idx + 1).context("Missing profile name")?;
            }
            "--release" => build_type_dir = "release",
            "--target" => {
                // When cross compiling we should generate the correct bundle type
                cross_compile_target = Some(
                    args.get(arg_idx + 1)
                        .context("Missing cross-compile target")?
                        .to_owned(),
                );
            }
            arg if arg.starts_with("--profile=") => {
                build_type_dir = arg
                    .strip_prefix("--profile=")
                    .context("Missing profile name")?;
            }
            arg if arg.starts_with("--target=") => {
                cross_compile_target = Some(
                    arg.strip_prefix("--target=")
                        .context("Missing cross-compile target")?
                        .to_owned(),
                );
            }
            _ => (),
        }
    }

    // We can bundle both library targets (for plugins) and binary targets (for standalone
    // applications)
    if universal {
        let x86_64_target_base =
            target_base(target_dir, Some("x86_64-apple-darwin"))?.join(build_type_dir);
        let x86_64_bin_path = x86_64_target_base.join(binary_basename(
            package,
            CompilationTarget::MacOS(Architecture::X86_64),
        ));
        let x86_64_lib_path = x86_64_target_base.join(library_basename(
            package,
            CompilationTarget::MacOS(Architecture::X86_64),
        ));

        let aarch64_target_base =
            target_base(target_dir, Some("aarch64-apple-darwin"))?.join(build_type_dir);
        let aarch64_bin_path = aarch64_target_base.join(binary_basename(
            package,
            CompilationTarget::MacOS(Architecture::AArch64),
        ));
        let aarch64_lib_path = aarch64_target_base.join(library_basename(
            package,
            CompilationTarget::MacOS(Architecture::AArch64),
        ));

        let build_bin = x86_64_bin_path.exists() && aarch64_bin_path.exists();
        let build_lib = x86_64_lib_path.exists() && aarch64_lib_path.exists();
        if !build_bin && !build_lib {
            anyhow::bail!("Could not find built libraries for universal build.");
        }

        eprintln!();
        if build_bin {
            bundle_binary(
                target_dir,
                package,
                &[&x86_64_bin_path, &aarch64_bin_path],
                CompilationTarget::MacOSUniversal,
            )?;
        }
        if build_lib {
            bundle_plugin(
                target_dir,
                package,
                &[&x86_64_lib_path, &aarch64_lib_path],
                CompilationTarget::MacOSUniversal,
            )?;
        }
    } else {
        let compilation_target = compilation_target(cross_compile_target.as_deref())?;
        let target_base =
            target_base(target_dir, cross_compile_target.as_deref())?.join(build_type_dir);
        let bin_path = target_base.join(binary_basename(package, compilation_target));
        let lib_path = target_base.join(library_basename(package, compilation_target));
        if !bin_path.exists() && !lib_path.exists() {
            anyhow::bail!(
                r#"Could not find a built library at '{}'.

Hint: Maybe you forgot to add:

[lib]
crate-type = ["cdylib"]

to your Cargo.toml file?"#,
                lib_path.display()
            );
        }

        eprintln!();
        if bin_path.exists() {
            bundle_binary(target_dir, package, &[&bin_path], compilation_target)?;
        }
        if lib_path.exists() {
            bundle_plugin(target_dir, package, &[&lib_path], compilation_target)?;
        }
    }

    Ok(())
}

/// Bundle a standalone target. If `bin_path` contains more than one path, then the binaries will be
/// combined into a single binary using a method that depends on the compilation target. For
/// universal macOS builds this uses lipo.
fn bundle_binary(
    target_dir: &Path,
    package: &str,
    bin_paths: &[&Path],
    compilation_target: CompilationTarget,
) -> Result<()> {
    let bundle_home_dir = bundle_home(target_dir);
    let bundle_name = match load_bundler_config()?.and_then(|c| c.get(package).cloned()) {
        Some(PackageConfig { name: Some(name) }) => name,
        _ => package.to_string(),
    };

    // On MacOS the standalone target needs to be in a bundle
    let standalone_bundle_binary_name =
        standalone_bundle_binary_name(&bundle_name, compilation_target);
    let standalone_binary_path = bundle_home_dir.join(&standalone_bundle_binary_name);

    fs::create_dir_all(standalone_binary_path.parent().unwrap())
        .context("Could not create standalone bundle directory")?;
    util::reflink_or_combine(bin_paths, &standalone_binary_path, compilation_target)
        .context("Could not create standalone bundle")?;

    // FIXME: The reflink crate seems to sometime strip away the executable bit, so we need to help
    //        it a little here
    #[cfg(unix)]
    if let Ok(metadata) = fs::metadata(&standalone_binary_path) {
        // These are the executable bits
        let mut permissions = metadata.permissions();
        permissions.set_mode(permissions.mode() | 0b0001001001);

        fs::set_permissions(&standalone_binary_path, permissions).with_context(|| {
            format!(
                "Could not make '{}' executable",
                standalone_binary_path.display()
            )
        })?;
    }

    let standalone_bundle_home = bundle_home_dir.join(
        Path::new(&standalone_bundle_binary_name)
            .components()
            .next()
            .expect("Malformed standalone binary path"),
    );
    maybe_create_macos_bundle_metadata(
        package,
        &bundle_name,
        &standalone_bundle_home,
        compilation_target,
        BundleType::Binary,
    )?;
    maybe_codesign(&standalone_bundle_home, compilation_target);

    eprintln!(
        "Created a standalone bundle at '{}'",
        standalone_bundle_home.display()
    );

    Ok(())
}

/// Bundle all plugin targets for a plugin library. If `lib_path` contains more than one path, then
/// the libraries will be combined into a single library using a method that depends on the
/// compilation target. For universal macOS builds this uses lipo.
fn bundle_plugin(
    target_dir: &Path,
    package: &str,
    lib_paths: &[&Path],
    compilation_target: CompilationTarget,
) -> Result<()> {
    let bundle_home_dir = bundle_home(target_dir);
    let bundle_name = match load_bundler_config()?.and_then(|c| c.get(package).cloned()) {
        Some(PackageConfig { name: Some(name) }) => name,
        _ => package.to_string(),
    };

    // We'll detect the plugin formats supported by the plugin binary and create bundled accordingly.
    // If `lib_path` contains paths to multiple plugins that need to be combined into a macOS
    // universal binary, then we'll assume all of them export the same symbols and only check the
    // first one.
    let first_lib_path = lib_paths.first().context("Empty library paths slice")?;

    let bundle_clap = symbols::exported(first_lib_path, "clap_entry")
        .with_context(|| format!("Could not parse '{}'", first_lib_path.display()))?;
    // We'll ignore the platform-specific entry points for VST2 plugins since there's no reason to
    // create a new Rust VST2 plugin that doesn't work in modern DAWs
    // NOTE: NIH-plug does not support VST2, but we'll support bundling VST2 plugins anyways because
    //       this bundler can also be used standalone.
    let bundle_vst2 = symbols::exported(first_lib_path, "VSTPluginMain")
        .with_context(|| format!("Could not parse '{}'", first_lib_path.display()))?;
    let bundle_vst3 = symbols::exported(first_lib_path, "GetPluginFactory")
        .with_context(|| format!("Could not parse '{}'", first_lib_path.display()))?;
    let bundle_auv3 = symbols::exported(first_lib_path, "plugin_create")
        .with_context(|| format!("Could not parse '{}'", first_lib_path.display()))?;
    let bundled_plugin = bundle_clap || bundle_vst2 || bundle_vst3 || bundle_auv3;

    if bundle_clap {
        let clap_bundle_library_name = clap_bundle_library_name(&bundle_name, compilation_target);
        let clap_lib_path = bundle_home_dir.join(&clap_bundle_library_name);

        fs::create_dir_all(clap_lib_path.parent().unwrap())
            .context("Could not create CLAP bundle directory")?;
        util::reflink_or_combine(lib_paths, &clap_lib_path, compilation_target)
            .context("Could not create CLAP bundle")?;

        // In contrast to VST3, CLAP only uses bundles on macOS, so we'll just take the first
        // component of the library name instead
        let clap_bundle_home = bundle_home_dir.join(
            Path::new(&clap_bundle_library_name)
                .components()
                .next()
                .expect("Malformed CLAP library path"),
        );
        maybe_create_macos_bundle_metadata(
            package,
            &bundle_name,
            &clap_bundle_home,
            compilation_target,
            BundleType::Plugin,
        )?;
        maybe_codesign(&clap_bundle_home, compilation_target);

        eprintln!("Created a CLAP bundle at '{}'", clap_bundle_home.display());
    }
    if bundle_vst2 {
        let vst2_bundle_library_name = vst2_bundle_library_name(&bundle_name, compilation_target);
        let vst2_lib_path = bundle_home_dir.join(&vst2_bundle_library_name);

        fs::create_dir_all(vst2_lib_path.parent().unwrap())
            .context("Could not create VST2 bundle directory")?;
        util::reflink_or_combine(lib_paths, &vst2_lib_path, compilation_target)
            .context("Could not create VST2 bundle")?;

        // VST2 only uses bundles on macOS, so we'll just take the first component of the library
        // name instead
        let vst2_bundle_home = bundle_home_dir.join(
            Path::new(&vst2_bundle_library_name)
                .components()
                .next()
                .expect("Malformed VST2 library path"),
        );
        maybe_create_macos_bundle_metadata(
            package,
            &bundle_name,
            &vst2_bundle_home,
            compilation_target,
            BundleType::Plugin,
        )?;
        maybe_codesign(&vst2_bundle_home, compilation_target);

        eprintln!("Created a VST2 bundle at '{}'", vst2_bundle_home.display());
    }
    if bundle_vst3 {
        let vst3_lib_path =
            bundle_home_dir.join(vst3_bundle_library_name(&bundle_name, compilation_target));

        fs::create_dir_all(vst3_lib_path.parent().unwrap())
            .context("Could not create VST3 bundle directory")?;
        util::reflink_or_combine(lib_paths, &vst3_lib_path, compilation_target)
            .context("Could not create VST3 bundle")?;

        let vst3_bundle_home = vst3_lib_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        maybe_create_macos_bundle_metadata(
            package,
            &bundle_name,
            vst3_bundle_home,
            compilation_target,
            BundleType::Plugin,
        )?;
        maybe_codesign(vst3_bundle_home, compilation_target);

        eprintln!("Created a VST3 bundle at '{}'", vst3_bundle_home.display());
    }
    if bundle_auv3 {
        let auv3_bundle_library_name = auv3_bundle_library_name(&bundle_name, compilation_target);
        let auv3_lib_path = bundle_home_dir.join(&auv3_bundle_library_name);

        fs::create_dir_all(auv3_lib_path.parent().unwrap())
            .context("Could not create AUv3 bundle directory")?;
        util::reflink_or_combine(lib_paths, &auv3_lib_path, compilation_target)
            .context("Could not create AUv3 bundle")?;

        let auv3_bundle_home = bundle_home_dir.join(
            Path::new(&auv3_bundle_library_name)
                .components()
                .next()
                .expect("Malformed AUv3 library path"),
        );

        // Generate AUv3-specific Info.plist with plugin metadata
        // For now, use hardcoded values - in a full implementation, these would be extracted
        // from the plugin library using FFI calls
        generate_auv3_infoplist(
            package,
            &bundle_name,
            &auv3_bundle_home,
            "Test Gain AUv3",  // plugin_name
            "NIH-Plug",        // plugin_vendor
            "1.0.0",           // plugin_version
            "https://github.com/robbert-vdh/nih-plug", // plugin_url
            "info@example.com", // plugin_email
            0x61756D75,        // au_type: 'aumu' (Audio Unit Music Effect)
            0x4E706C67,        // au_subtype: 'Nplg' (NIH-Plug identifier - mixed case)
            0x4E504C47,        // au_manufacturer: 'NPLG' (NIH-Plug manufacturer)
        )?;

        maybe_codesign(&auv3_bundle_home, compilation_target);

        eprintln!("Created an AUv3 bundle at '{}'", auv3_bundle_home.display());
    }
    if !bundled_plugin {
        eprintln!("Not creating any plugin bundles because the package does not export any plugins")
    }

    Ok(())
}

/// This lists the packages configured in `bundler.toml`. This is only used as part of the CI when
/// bundling plugins.
pub fn list_known_packages() -> Result<()> {
    if let Some(config) = load_bundler_config()? {
        for package in config.keys() {
            println!("{package}");
        }
    }

    Ok(())
}

/// Load the `bundler.toml` file, if it exists. If it does exist but it cannot be parsed, then this
/// will return an error.
fn load_bundler_config() -> Result<Option<BundlerConfig>> {
    // We're already in the project root
    let bundler_config_path = Path::new("bundler.toml");
    if !bundler_config_path.exists() {
        return Ok(None);
    }

    let result = toml::from_str(
        &fs::read_to_string(bundler_config_path)
            .with_context(|| format!("Could not read '{}'", bundler_config_path.display()))?,
    )
    .with_context(|| format!("Could not parse '{}'", bundler_config_path.display()))?;

    Ok(Some(result))
}

/// Split the `xtask bundle` arguments into a list of packages and a list of other arguments. The
/// package vector either contains just the first argument, or if the arguments iterator starts with
/// one or more occurences of `-p <package>` then this will contain all those packages.
fn split_bundle_args(
    args: impl Iterator<Item = String>,
    usage_string: &str,
) -> Result<(Vec<String>, Vec<String>)> {
    let mut args = args.peekable();
    let mut packages = Vec::new();
    if args.peek().map(|s| s.as_str()) == Some("-p") {
        while args.peek().map(|s| s.as_str()) == Some("-p") {
            packages.push(
                args.nth(1)
                    .with_context(|| format!("Missing package name after -p\n\n{usage_string}"))?,
            );
        }
    } else {
        packages.push(
            args.next()
                .with_context(|| format!("Missing package name\n\n{usage_string}"))?,
        );
    };
    let other_args: Vec<_> = args.collect();

    Ok((packages, other_args))
}

/// The target we're compiling for. This is used to determine the paths and options for creating
/// plugin bundles.
fn compilation_target(cross_compile_target: Option<&str>) -> Result<CompilationTarget> {
    match cross_compile_target {
        Some("i686-unknown-linux-gnu") => Ok(CompilationTarget::Linux(Architecture::X86)),
        Some("i686-apple-darwin") => Ok(CompilationTarget::MacOS(Architecture::X86)),
        Some("i686-pc-windows-gnu") | Some("i686-pc-windows-msvc") => {
            Ok(CompilationTarget::Windows(Architecture::X86))
        }
        Some("x86_64-unknown-linux-gnu") => Ok(CompilationTarget::Linux(Architecture::X86_64)),
        Some("x86_64-apple-darwin") => Ok(CompilationTarget::MacOS(Architecture::X86_64)),
        Some("x86_64-pc-windows-gnu") | Some("x86_64-pc-windows-msvc") => {
            Ok(CompilationTarget::Windows(Architecture::X86_64))
        }
        Some("aarch64-unknown-linux-gnu") => Ok(CompilationTarget::Linux(Architecture::AArch64)),
        Some("aarch64-apple-darwin") => Ok(CompilationTarget::MacOS(Architecture::AArch64)),
        Some("aarch64-pc-windows-gnu") | Some("aarch64-pc-windows-msvc") => {
            Ok(CompilationTarget::Windows(Architecture::AArch64))
        }
        Some(target) => anyhow::bail!("Unhandled cross-compilation target: {}", target),
        None => {
            #[cfg(target_arch = "x86")]
            let architecture = Architecture::X86;
            #[cfg(target_arch = "x86_64")]
            let architecture = Architecture::X86_64;
            #[cfg(target_arch = "aarch64")]
            let architecture = Architecture::AArch64;
            #[cfg(target_arch = "riscv64")]
            let architecture = Architecture::RISCV64;

            #[cfg(target_os = "linux")]
            return Ok(CompilationTarget::Linux(architecture));
            #[cfg(target_os = "macos")]
            return Ok(CompilationTarget::MacOS(architecture));
            #[cfg(target_os = "windows")]
            return Ok(CompilationTarget::Windows(architecture));
        }
    }
}

/// The directory bundled plugins should be written to.
fn bundle_home(target_directory: &Path) -> PathBuf {
    target_directory.join("bundled")
}

/// The base directory for the compiled binaries. This does not use [`CompilationTarget`] as we need
/// to be able to differentiate between native and cross-compilation.
fn target_base(target_directory: &Path, cross_compile_target: Option<&str>) -> Result<PathBuf> {
    match cross_compile_target {
        // Unhandled targets will already be handled in `compilation_target`
        Some(target) => Ok(target_directory.join(target)),
        None => Ok(target_directory.to_owned()),
    }
}

/// The file name of the compiled library for a binary crate.
fn binary_basename(package: &str, target: CompilationTarget) -> String {
    // Cargo will replace dashes with underscores
    let bin_name = package.replace('-', "_");

    match target {
        CompilationTarget::Linux(_)
        | CompilationTarget::MacOS(_)
        | CompilationTarget::MacOSUniversal => bin_name,
        CompilationTarget::Windows(_) => format!("{bin_name}.exe"),
    }
}

/// The file name of the compiled library for a `cdylib` crate.
fn library_basename(package: &str, target: CompilationTarget) -> String {
    // Cargo will replace dashes with underscores
    let lib_name = package.replace('-', "_");

    match target {
        CompilationTarget::Linux(_) => format!("lib{lib_name}.so"),
        CompilationTarget::MacOS(_) | CompilationTarget::MacOSUniversal => {
            format!("lib{lib_name}.dylib")
        }
        CompilationTarget::Windows(_) => format!("{lib_name}.dll"),
    }
}

/// The filename of the binary target. On macOS this is part of a bundle.
fn standalone_bundle_binary_name(package: &str, target: CompilationTarget) -> String {
    match target {
        CompilationTarget::Linux(_) => package.to_owned(),
        CompilationTarget::MacOS(_) | CompilationTarget::MacOSUniversal => {
            format!("{package}.app/Contents/MacOS/{package}")
        }
        CompilationTarget::Windows(_) => format!("{package}.exe"),
    }
}

/// The filename of the CLAP plugin for Linux and Windows, or the full path to the library file
/// inside of a CLAP bundle on macOS.
fn clap_bundle_library_name(package: &str, target: CompilationTarget) -> String {
    match target {
        CompilationTarget::Linux(_) | CompilationTarget::Windows(_) => format!("{package}.clap"),
        CompilationTarget::MacOS(_) | CompilationTarget::MacOSUniversal => {
            format!("{package}.clap/Contents/MacOS/{package}")
        }
    }
}

/// On Linux and Windows VST2 plugins are regular library files, and on macOS they are put in a
/// bundle.
fn vst2_bundle_library_name(package: &str, target: CompilationTarget) -> String {
    match target {
        CompilationTarget::Linux(_) => format!("{package}.so"),
        CompilationTarget::MacOS(_) | CompilationTarget::MacOSUniversal => {
            format!("{package}.vst/Contents/MacOS/{package}")
        }
        CompilationTarget::Windows(_) => format!("{package}.dll"),
    }
}

/// The full path to the library file inside of a VST3 bundle, including the leading `.vst3`
/// directory.
///
/// See <https://developer.steinberg.help/display/VST/Plug-in+Format+Structure>.
fn vst3_bundle_library_name(package: &str, target: CompilationTarget) -> String {
    match target {
        CompilationTarget::Linux(Architecture::X86) => {
            format!("{package}.vst3/Contents/i386-linux/{package}.so")
        }
        CompilationTarget::Linux(Architecture::X86_64) => {
            format!("{package}.vst3/Contents/x86_64-linux/{package}.so")
        }
        CompilationTarget::Linux(Architecture::RISCV64) => {
            format!("{package}.vst3/Contents/riscv64-linux/{package}.so")
        }
        CompilationTarget::Linux(Architecture::AArch64) => {
            format!("{package}.vst3/Contents/aarch64-linux/{package}.so")
        }
        CompilationTarget::MacOS(_) | CompilationTarget::MacOSUniversal => {
            format!("{package}.vst3/Contents/MacOS/{package}")
        }
        CompilationTarget::Windows(Architecture::X86) => {
            format!("{package}.vst3/Contents/x86-win/{package}.vst3")
        }
        CompilationTarget::Windows(Architecture::X86_64) => {
            format!("{package}.vst3/Contents/x86_64-win/{package}.vst3")
        }
        CompilationTarget::Windows(Architecture::AArch64) => {
            format!("{package}.vst3/Contents/arm_64-win/{package}.vst3")
        }
        CompilationTarget::Windows(Architecture::RISCV64) => {
            panic!("riscv64 are not supported by windows currently!")
        }
    }
}

/// The full path to the library file inside of an AUv3 bundle, including the leading `.appex`
/// directory.
///
/// AUv3 plugins are macOS-only and use the App Extension (.appex) format.
fn auv3_bundle_library_name(package: &str, target: CompilationTarget) -> String {
    match target {
        CompilationTarget::MacOS(_) | CompilationTarget::MacOSUniversal => {
            format!("{package}.appex/Contents/MacOS/{package}")
        }
        _ => {
            // AUv3 is macOS-only, so we don't support other platforms
            panic!("AUv3 plugins are only supported on macOS")
        }
    }
}

/// If compiling for macOS, create all of the bundl-y stuff Steinberg and Apple require you to have.
///
/// This still requires you to move the dylib file to `{bundle_home}/Contents/macOS/{package}`
/// yourself first.
pub fn maybe_create_macos_bundle_metadata(
    package: &str,
    display_name: &str,
    bundle_home: &Path,
    target: CompilationTarget,
    bundle_type: BundleType,
) -> Result<()> {
    if !matches!(
        target,
        CompilationTarget::MacOS(_) | CompilationTarget::MacOSUniversal
    ) {
        return Ok(());
    }

    let package_type = match bundle_type {
        BundleType::Plugin => "BNDL",
        BundleType::Binary => "APPL",
    };

    // TODO: May want to add bundler.toml fields for the identifier, version and signature at some
    //       point.
    fs::write(
        bundle_home.join("Contents").join("PkgInfo"),
        format!("{package_type}????"),
    )
    .context("Could not create PkgInfo file")?;
    fs::write(
        bundle_home.join("Contents").join("Info.plist"),
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>

<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist>
  <dict>
    <key>CFBundleExecutable</key>
    <string>{display_name}</string>
    <key>CFBundleIconFile</key>
    <string></string>
    <key>CFBundleIdentifier</key>
    <string>com.nih-plug.{package}</string>
    <key>CFBundleName</key>
    <string>{display_name}</string>
    <key>CFBundleDisplayName</key>
    <string>{display_name}</string>
    <key>CFBundlePackageType</key>
    <string>{package_type}</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>NSHumanReadableCopyright</key>
    <string></string>
    <key>NSHighResolutionCapable</key>
    <true/>
  </dict>
</plist>
"#),
    )
    .context("Could not create Info.plist file")?;

    Ok(())
}

/// Generate Info.plist for AUv3 app extension with plugin metadata.
///
/// This creates a proper Info.plist for Audio Unit v3 app extensions that includes
/// the AudioComponents array with plugin registration information.
pub fn generate_auv3_infoplist(
    package: &str,
    display_name: &str,
    bundle_home: &Path,
    plugin_name: &str,
    plugin_vendor: &str,
    plugin_version: &str,
    plugin_url: &str,
    plugin_email: &str,
    au_type: u32,
    au_subtype: u32,
    au_manufacturer: u32,
) -> Result<()> {
    if !bundle_home.join("Contents").exists() {
        fs::create_dir_all(bundle_home.join("Contents"))
            .context("Could not create Contents directory")?;
    }

    // Convert 4-byte codes to strings for display
    let au_type_str = format!("{:08x}", au_type);
    let au_subtype_str = format!("{:08x}", au_subtype);
    let au_manufacturer_str = format!("{:08x}", au_manufacturer);

    // Convert hex strings to 4-character codes
    let au_type_code = format!("{}{}{}{}", 
        char::from_u32(au_type >> 24 & 0xFF).unwrap_or('?'),
        char::from_u32(au_type >> 16 & 0xFF).unwrap_or('?'),
        char::from_u32(au_type >> 8 & 0xFF).unwrap_or('?'),
        char::from_u32(au_type & 0xFF).unwrap_or('?')
    );
    
    let au_subtype_code = format!("{}{}{}{}", 
        char::from_u32(au_subtype >> 24 & 0xFF).unwrap_or('?'),
        char::from_u32(au_subtype >> 16 & 0xFF).unwrap_or('?'),
        char::from_u32(au_subtype >> 8 & 0xFF).unwrap_or('?'),
        char::from_u32(au_subtype & 0xFF).unwrap_or('?')
    );
    
    let au_manufacturer_code = format!("{}{}{}{}", 
        char::from_u32(au_manufacturer >> 24 & 0xFF).unwrap_or('?'),
        char::from_u32(au_manufacturer >> 16 & 0xFF).unwrap_or('?'),
        char::from_u32(au_manufacturer >> 8 & 0xFF).unwrap_or('?'),
        char::from_u32(au_manufacturer & 0xFF).unwrap_or('?')
    );

    let info_plist_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>$(DEVELOPMENT_LANGUAGE)</string>
    <key>CFBundleDisplayName</key>
    <string>{display_name}</string>
    <key>CFBundleExecutable</key>
    <string>$(EXECUTABLE_NAME)</string>
    <key>CFBundleIdentifier</key>
    <string>$(PRODUCT_BUNDLE_IDENTIFIER)</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>$(PRODUCT_NAME)</string>
    <key>CFBundlePackageType</key>
    <string>$(PRODUCT_BUNDLE_PACKAGE_TYPE)</string>
    <key>CFBundleShortVersionString</key>
    <string>{plugin_version}</string>
    <key>CFBundleVersion</key>
    <string>{plugin_version}</string>
    <key>LSMinimumSystemVersion</key>
    <string>$(MACOSX_DEPLOYMENT_TARGET)</string>
    <key>NSExtension</key>
    <dict>
        <key>NSExtensionPointIdentifier</key>
        <string>com.apple.AudioUnit-UI</string>
        <key>NSExtensionPrincipalClass</key>
        <string>$(PRODUCT_MODULE_NAME).AUAudioUnit</string>
    </dict>
    <key>AudioComponents</key>
    <array>
        <dict>
            <key>type</key>
            <string>{au_type_code}</string>
            <key>subtype</key>
            <string>{au_subtype_code}</string>
            <key>manufacturer</key>
            <string>{au_manufacturer_code}</string>
            <key>name</key>
            <string>{plugin_name}</string>
            <key>description</key>
            <string>{plugin_name} - {plugin_vendor}</string>
            <key>version</key>
            <integer>1</integer>
            <key>sandboxSafe</key>
            <true/>
            <key>hasCustomView</key>
            <true/>
        </dict>
    </array>
</dict>
</plist>
"#);

    fs::write(
        bundle_home.join("Contents").join("Info.plist"),
        info_plist_content,
    )
    .context("Could not create AUv3 Info.plist file")?;

    Ok(())
}

/// If compiling for macOS, try to self-sign the bundle at the given path. This shouldn't be
/// necessary, but AArch64 macOS is stricter about these things and sometimes self built plugins may
/// not load otherwise. Presumably in combination with hardened runtimes.
///
/// If the codesigning command could not be run then this merely prints a warning.
pub fn maybe_codesign(bundle_home: &Path, target: CompilationTarget) {
    if !matches!(
        target,
        CompilationTarget::MacOS(_) | CompilationTarget::MacOSUniversal
    ) {
        return;
    }

    // Clean extended attributes that might prevent signing
    let _ = Command::new("xattr")
        .arg("-cr")
        .arg(bundle_home)
        .status();

    // Remove .DS_Store files
    let _ = Command::new("find")
        .arg(bundle_home)
        .arg("-name")
        .arg(".DS_Store")
        .arg("-delete")
        .status();

    let success = Command::new("codesign")
        .arg("-f")
        .arg("-s")
        .arg("-")
        .arg(bundle_home)
        .status()
        .is_ok();
    if !success {
        eprintln!(
            "WARNING: Could not self-sign '{}', it may fail to run depending on the environment",
            bundle_home.display()
        )
    }
}

/// Build the AUv3 Swift app extension using Xcode.
/// This function:
/// 1. Runs the build_rust.sh script to prepare the Swift project
/// 2. Uses xcodebuild to build the Swift app extension
/// 3. Handles errors and provides clear feedback
fn xcode_build_auv3(package: &str) -> Result<()> {
    let swift_dir = Path::new("src/wrapper/auv3/swift");
    
    // Check if the Swift directory exists
    if !swift_dir.exists() {
        anyhow::bail!(
            "Swift directory not found at '{}'. Make sure you're in the NIH-plug workspace root.",
            swift_dir.display()
        );
    }

    // Check if the Xcode project exists
    let xcode_project = swift_dir.join("NIHPlugAUv3.xcodeproj");
    if !xcode_project.exists() {
        anyhow::bail!(
            "Xcode project not found at '{}'. Make sure the AUv3 Swift project is set up.",
            xcode_project.display()
        );
    }

    // Check if the build script exists
    let build_script = swift_dir.join("build_rust.sh");
    if !build_script.exists() {
        anyhow::bail!(
            "Build script not found at '{}'. Make sure the AUv3 build script is present.",
            build_script.display()
        );
    }

    eprintln!("Building AUv3 Swift app extension for package '{}'...", package);

    // Step 1: Run the build_rust.sh script to prepare the Swift project
    eprintln!("Step 1: Building Rust library and preparing Swift project...");
    let build_script_status = Command::new("bash")
        .arg("build_rust.sh")
        .current_dir(swift_dir)
        .status()
        .context("Could not run build_rust.sh script")?;

    if !build_script_status.success() {
        anyhow::bail!("build_rust.sh script failed with exit code: {:?}", build_script_status.code());
    }

    // Step 2: Build the Swift app extension using xcodebuild
    eprintln!("Step 2: Building Swift app extension with xcodebuild...");
    let xcodebuild_status = Command::new("xcodebuild")
        .arg("-project")
        .arg("NIHPlugAUv3.xcodeproj")
        .arg("-target")
        .arg("NIHPlugAUv3")
        .arg("-configuration")
        .arg("Debug")
        .arg("build")
        .current_dir(swift_dir)
        .status()
        .context("Could not run xcodebuild")?;

    if !xcodebuild_status.success() {
        anyhow::bail!("xcodebuild failed with exit code: {:?}", xcodebuild_status.code());
    }

    // Step 3: Check if the app extension was built successfully
    let appex_path = swift_dir.join("build/Debug/NIHPlugAUv3.appex");
    if !appex_path.exists() {
        anyhow::bail!(
            "App extension not found at '{}'. The build may have failed.",
            appex_path.display()
        );
    }

    eprintln!("✅ Successfully built AUv3 app extension at '{}'", appex_path.display());
    eprintln!("The app extension is ready for testing in Logic Pro or GarageBand.");

    Ok(())
}
