use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

const SPECIAL_NAMES: [&str; 3] = ["icon", "light", "night"];
const SUPPORTED_EXTENSIONS: [&str; 6] = ["jpg", "jpeg", "webp", "gif", "bmp", "tiff"];

fn main() {
    println!("cargo:rerun-if-changed=public");
    if let Err(err) = convert_special_images() {
        println!("cargo:warning=build.rs image conversion failed: {err}");
    }
}

fn convert_special_images() -> Result<(), Box<dyn std::error::Error>> {
    let public_dir = Path::new("public");
    if !public_dir.exists() {
        return Ok(());
    }

    for name in SPECIAL_NAMES {
        let target_png = public_dir.join(format!("{name}.png"));
        if target_png.exists() {
            continue;
        }

        let Some(source) = find_first_source(public_dir, name) else {
            continue;
        };

        if let Err(err) = convert_to_png(&source, &target_png) {
            println!(
                "cargo:warning=failed to convert {source:?} to {target_png:?}: {err}"
            );
        }
    }

    Ok(())
}

fn find_first_source(public_dir: &Path, name: &str) -> Option<PathBuf> {
    for ext in SUPPORTED_EXTENSIONS {
        let candidate = public_dir.join(format!("{name}.{ext}"));
        if candidate.exists() {
            return Some(candidate);
        }
    }

    read_first_matching_file(public_dir, name)
}

fn read_first_matching_file(public_dir: &Path, name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(public_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_stem().and_then(OsStr::to_str) != Some(name) {
            continue;
        }
        let extension = path.extension().and_then(OsStr::to_str).map(str::to_lowercase);
        if let Some(extension) = extension {
            if extension != "png" && SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
                return Some(path);
            }
            if extension != "png" {
                println!(
                    "cargo:warning=unsupported image extension '{extension}' for {name}"
                );
            }
        }
    }

    None
}

fn convert_to_png(source: &Path, target: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let image = image::open(source)?;
    let mut output = fs::File::create(target)?;
    image.write_to(&mut output, image::ImageFormat::Png)?;
    Ok(())
}
