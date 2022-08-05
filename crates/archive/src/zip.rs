use moon_error::{map_io_to_fs_error, MoonError};
use moon_logger::{color, trace};
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

fn zip_file(
    archive: &mut ZipWriter<File>,
    file: &Path,
    base_prefix: &Option<&str>,
) -> Result<(), MoonError> {
    let file_name = file.file_name().unwrap().to_str().unwrap();
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755);

    archive
        .start_file(
            match base_prefix {
                Some(prefix) => format!("{}/{}", prefix, file_name),
                None => file_name.to_owned(),
            },
            options,
        )
        .unwrap();

    archive.write_all(&fs::read(file)?).unwrap();

    Ok(())
}

// fn zip_dir(dir: &Path) -> Result<(), MoonError> {
//     for entry in fs::read_dir(dir)? {}

//     Ok(())
// }

#[track_caller]
pub fn zip<I: AsRef<Path>, O: AsRef<Path>>(
    input_src: I,
    output_file: O,
    base_prefix: Option<&str>,
) -> Result<(), MoonError> {
    let input_src = input_src.as_ref();
    let output_file = output_file.as_ref();

    trace!(
        target: "moon:archive:zip",
        "Packing zip archive with {} to {}",
        color::path(input_src),
        color::path(output_file),
    );

    // Create .zip
    let zip =
        File::create(output_file).map_err(|e| map_io_to_fs_error(e, output_file.to_path_buf()))?;

    // Add the files to the archive
    let mut archive = ZipWriter::new(zip);

    if input_src.is_file() {
        zip_file(&mut archive, input_src, &base_prefix)?;
    } else {
    }

    archive.finish().unwrap();

    Ok(())
}

#[track_caller]
pub fn unzip<I: AsRef<Path>, O: AsRef<Path>>(
    input_file: I,
    output_dir: O,
    remove_prefix: Option<&str>,
) -> Result<(), MoonError> {
    let input_file = input_file.as_ref();
    let output_dir = output_dir.as_ref();

    trace!(
        target: "moon:archive:zip",
        "Unzipping zip archive {} to {}",
        color::path(input_file),
        color::path(output_dir),
    );

    if !output_dir.exists() {
        fs::create_dir_all(output_dir)
            .map_err(|e| map_io_to_fs_error(e, output_dir.to_path_buf()))?;
    }

    // Open .zip file
    let zip =
        File::open(input_file).map_err(|e| map_io_to_fs_error(e, input_file.to_path_buf()))?;

    // Unpack the archive into the output dir
    let mut archive = ZipArchive::new(zip).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let mut path = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        // Remove the prefix
        if let Some(prefix) = remove_prefix {
            if path.starts_with(prefix) {
                path = path.strip_prefix(&prefix).unwrap().to_owned();
            }
        }

        let output_path = output_dir.join(&path);
        let handle_error = |e: io::Error| map_io_to_fs_error(e, output_path.to_path_buf());

        // Create parent dirs
        if let Some(parent_dir) = output_path.parent() {
            if !parent_dir.exists() {
                fs::create_dir_all(parent_dir)
                    .map_err(|e| map_io_to_fs_error(e, parent_dir.to_path_buf()))?;
            }
        }

        // If a file, copy it to the output dir
        if file.is_file() {
            let mut out = File::create(&output_path).map_err(handle_error)?;

            io::copy(&mut file, &mut out).map_err(handle_error)?;
        }

        // Update permissions when on a nix machine
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&output_path, fs::Permissions::from_mode(mode))
                    .map_err(handle_error)?;
            }
        }
    }

    Ok(())
}
