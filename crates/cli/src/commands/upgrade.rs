use crate::helpers::{create_progress_bar, AnyError};
use bytes::Buf;
use itertools::Itertools;
use moon_launchpad::check_version;
use moon_logger::error;
use moon_utils::{
    fs::{create_dir_all, rename},
    semver::Version,
};
use proto::ProtoError;
use std::{env, fs::File, io::copy, path::Component};

pub async fn upgrade() -> Result<(), AnyError> {
    let version = env!("CARGO_PKG_VERSION");
    let version_check = check_version(version).await;

    let new_version = match version_check {
        Ok((newer_version, _)) if Version::parse(&newer_version)? > Version::parse(version)? => {
            newer_version
        }
        Ok(_) => {
            println!("You're already on the latest version of moon!");
            return Ok(());
        }
        Err(err) => {
            error!("Failed to get current version of the cli from remote: {err}");
            return Err(err);
        }
    };

    let target = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", arch) => {
            // Run ldd to check if we're running on musl
            let output = std::process::Command::new("ldd")
                .arg("--version")
                .output()?;
            let output = String::from_utf8(output.stdout)?;
            let libc = match output.contains("musl") {
                true => "musl",
                false => "gnu",
            };
            format!("moon-{arch}-unknown-linux-{libc}")
        }
        ("macos", arch) => format!("moon-{arch}-apple-darwin"),
        ("windows", "x86_64") => "moon-x86_64-pc-windows-msvc.exe".to_owned(),
        (_, arch) => {
            return Err(
                ProtoError::UnsupportedArchitecture("moon".to_owned(), arch.to_owned()).into(),
            )
        }
    };

    let bin_path = env::current_exe()?;

    // We can only upgrade moon if it's installed under .moon
    let upgradeable = bin_path
        .components()
        .contains(&Component::Normal(".moon".as_ref()));

    if !upgradeable {
        return Err(format!(
            "Moon can only upgrade itself from the default .moon directory. \n\
            Moon is currently installed at: {}",
            bin_path.to_string_lossy()
        )
        .into());
    }

    let done = create_progress_bar(format!("Upgrading moon to version {new_version}..."));

    // Move the old binary to a versioned path
    let ver_path = bin_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(version)
        .join(bin_path.file_name().unwrap());
    create_dir_all(ver_path.parent().unwrap())?;
    rename(&bin_path, ver_path)?;

    let mut file = File::create(bin_path)?;

    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o755);
        file.set_permissions(perms)?;
    }

    let new_bin = reqwest::get(format!(
        "https://github.com/moonrepo/moon/releases/latest/download/{target}"
    ))
    .await?
    .bytes()
    .await?;

    copy(&mut new_bin.reader(), &mut file)?;

    done(
        format!("Successfully upgraded moon to version {new_version}"),
        true,
    );

    Ok(())
}
