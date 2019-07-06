use std::env;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry,WalkDir};
use zip::write::{FileOptions, ZipWriter};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let input_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap();

    // We have a virtual workspace, so we should go up a directory for all crates
    let mut root_dir = PathBuf::from(&input_dir);
    root_dir.pop();

    let mut out_zip = PathBuf::from(&out_dir);
    out_zip.push("source.zip");
    // This takes an additional 20 seconds, which is annoying in debug mode. As debug builds are
    // not supposed to be distributed, turn off source bundling in development.
    if profile == "release" {
        zip_sources(&root_dir, &out_zip).unwrap();
    } else {
        write_file(&out_dir, "source.zip", b"This is a debug build, no source code is present").unwrap();
    }

    write_file(&out_dir, "embed_source.rs", b"
        static SOURCE_ZIP: &'static [u8] = include_bytes!(\"source.zip\");

        fn download_source(_req: HttpRequest<AppState>) -> impl Responder {
            HttpResponse::Ok()
                .header(\"Content-Type\", \"application/zip\")
                .header(\"Content-Disposition\", \"attachment; filename=\\\"source.zip\\\"\")
                .body(SOURCE_ZIP)
        }
    ").unwrap();
}

/// Check if file should be included in the source zip.
///
/// Current blacklist:
/// - .env: could contain database credentials
/// - .git: contains version control secrets
/// - target: cargo output, would cause matroshka-doll issues
fn zip_file_filter(entry: &DirEntry) -> bool {
    let p = entry.file_name().to_str().unwrap();
    !(p.ends_with(".env") || p.starts_with("target") || p.starts_with(".git"))
}

/// Create a zip of all sources.
///
/// It walks the input_dir for all files, with a hardcoded blacklist of items not to include. All
/// files are written into a zip that is placed in output_file_location.
fn zip_sources(input_dir: &Path, output_file_location: &Path) -> Result<(), Error> {
    let output_file = File::create(output_file_location)?;
    let mut zip = ZipWriter::new(output_file);

    let zip_options = FileOptions::default();
    let mut buffer = Vec::new();

    for entry in WalkDir::new(input_dir).into_iter().filter_entry(zip_file_filter) {
        let real_entry = entry?;
        let path = real_entry.path();
        let zip_path = path.strip_prefix(input_dir).unwrap();
        if path.is_file() {
            println!("{}", zip_path.display());
            let mut file = File::open(path)?;
            file.read_to_end(&mut buffer)?;
            zip.start_file_from_path(zip_path, zip_options)?;
            zip.write_all(&*buffer)?;
        } else {
            println!("Not a file: {}", path.display());
        }
    }

    zip.finish()?;
    return Ok(());
}

/// Write a file with specified contents.
///
/// The string contents will be written to output_file in the directory output_dir.
fn write_file(output_dir: &str, output_file: &str, contents: &[u8]) -> Result<(), Error> {
    let mut output_path = PathBuf::from(output_dir);
    output_path.push(output_file);
    let mut file = File::create(&output_path)?;
    file.write_all(contents)?;
    Ok(())
}
