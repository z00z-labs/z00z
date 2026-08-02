//! Generate the sole bounded application action-basis manifest.

use std::{env, path::PathBuf};

use z00z_core::actions::LifecycleEffectV1;
use z00z_utils::codec::{AppWireCodec, AppWireEnvelope, AppWireField};
use z00z_utils::io::{destinations_alias_no_follow, sha256_256, to_lower_hex, write_file};

const SCHEMA: &[u8] = b"z00z.action-basis-manifest.v1";
const BACKEND_REVISION: &[u8] = b"z00z_core@0.2.0";
const ACTION_DESCRIPTOR_SOURCE: &[u8] = include_bytes!("../src/actions/action_descriptor.rs");
const BASIS_DOMAIN: &[u8] =
    b"z00z.action-basis.v1\0count=24\0no_state_change=explicitly_excluded\0";
const EXCLUSION: &[u8] = b"no_state_change=excluded";

fn main() {
    if let Err(error) = run() {
        eprintln!("z00z_action_basis_manifest: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let (output, digest_output) = parse_args()?;
    validate_destinations(&output, &digest_output)?;
    if LifecycleEffectV1::ATOMIC_BASIS.len() != 24 {
        return Err(format!(
            "atomic basis must contain exactly 24 effects, got {}",
            LifecycleEffectV1::ATOMIC_BASIS.len()
        ));
    }
    let mut basis_preimage = BASIS_DOMAIN.to_vec();
    let mut fields = vec![
        AppWireField::new(1, SCHEMA.to_vec()),
        AppWireField::new(2, BACKEND_REVISION.to_vec()),
        AppWireField::new(3, sha256_256(ACTION_DESCRIPTOR_SOURCE).to_vec()),
        AppWireField::new(5, 24_u16.to_be_bytes().to_vec()),
        AppWireField::new(6, EXCLUSION.to_vec()),
    ];
    for (index, effect) in LifecycleEffectV1::ATOMIC_BASIS.iter().copied().enumerate() {
        let name = effect.as_str().as_bytes();
        if name.len() > u8::MAX as usize {
            return Err("basis effect name exceeds manifest bound".to_string());
        }
        let mut entry = Vec::with_capacity(3 + name.len());
        entry.extend_from_slice(&(index as u16).to_be_bytes());
        entry.push(name.len() as u8);
        entry.extend_from_slice(name);
        basis_preimage.extend_from_slice(&entry);
        fields.push(AppWireField::new(100 + index as u16, entry));
    }
    fields.insert(
        3,
        AppWireField::new(4, sha256_256(&basis_preimage).to_vec()),
    );
    let bytes = AppWireCodec
        .encode(&AppWireEnvelope::v1(fields))
        .map_err(|error| error.to_string())?;
    let digest = sha256_256(&bytes);

    write_file(&output, &bytes).map_err(|error| error.to_string())?;
    write_file(
        &digest_output,
        format!("{}\n", to_lower_hex(&digest)).as_bytes(),
    )
    .map_err(|error| error.to_string())?;
    println!(
        "generated {} effects: {} ({})",
        LifecycleEffectV1::ATOMIC_BASIS.len(),
        output.display(),
        to_lower_hex(&digest)
    );
    Ok(())
}

fn parse_args() -> Result<(PathBuf, PathBuf), String> {
    let mut args = env::args_os().skip(1);
    let mut output = None;
    let mut digest = None;
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {}", flag.to_string_lossy()))?;
        match flag.to_string_lossy().as_ref() {
            "--output" => output = Some(PathBuf::from(value)),
            "--sha256" => digest = Some(PathBuf::from(value)),
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
    }
    Ok((
        output.ok_or_else(|| "--output is required".to_string())?,
        digest.ok_or_else(|| "--sha256 is required".to_string())?,
    ))
}

fn validate_destinations(output: &PathBuf, digest_output: &PathBuf) -> Result<(), String> {
    let output = normalize_destination(output)?;
    let digest_output = normalize_destination(digest_output)?;
    if output == digest_output
        || destinations_alias_no_follow(&output, &digest_output)
            .map_err(|error| error.to_string())?
    {
        return Err(format!(
            "manifest and digest destinations must differ: {}",
            output.display()
        ));
    }
    Ok(())
}

fn normalize_destination(path: &PathBuf) -> Result<PathBuf, String> {
    if path.as_os_str().is_empty() {
        return Err("output path must not be empty".to_string());
    }
    let absolute = if path.is_absolute() {
        path.clone()
    } else {
        env::current_dir()
            .map_err(|error| error.to_string())?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    return Err(format!(
                        "output path escapes its filesystem root: {}",
                        path.display()
                    ));
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_unchanged(path: &std::path::Path, expected: &[u8]) {
        assert_eq!(z00z_utils::io::read_file(path).unwrap(), expected);
    }

    #[test]
    fn same_destination_leaves_existing_file_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("manifest.bin");
        write_file(&destination, b"same-existing").unwrap();

        let error = validate_destinations(&destination, &destination).unwrap_err();

        assert!(error.contains("must differ"));
        assert_unchanged(&destination, b"same-existing");
    }

    #[test]
    fn colliding_destinations_leave_existing_file_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("manifest.bin");
        write_file(&destination, b"existing").unwrap();

        let alias = dir.path().join("nested/../manifest.bin");
        let error = validate_destinations(&destination, &alias).unwrap_err();

        assert!(error.contains("must differ"));
        assert_unchanged(&destination, b"existing");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_parent_alias_leaves_existing_file_unchanged() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        let alias = dir.path().join("alias");
        std::fs::create_dir(&real).unwrap();
        symlink(&real, &alias).unwrap();
        let output = real.join("same.bin");
        let digest = alias.join("same.bin");
        write_file(&output, b"symlink-existing").unwrap();

        let error = validate_destinations(&output, &digest).unwrap_err();

        assert!(error.contains("must differ"));
        assert_unchanged(&output, b"symlink-existing");
        assert_unchanged(&digest, b"symlink-existing");
    }

    #[test]
    fn hard_link_destinations_leave_existing_files_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("manifest.bin");
        let digest = dir.path().join("manifest.sha256");
        write_file(&output, b"hard-link-existing").unwrap();
        std::fs::hard_link(&output, &digest).unwrap();

        let error = validate_destinations(&output, &digest).unwrap_err();

        assert!(error.contains("must differ"));
        assert_unchanged(&output, b"hard-link-existing");
        assert_unchanged(&digest, b"hard-link-existing");
    }

    #[cfg(unix)]
    #[test]
    fn final_symlink_destination_is_rejected_without_following_it() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("manifest.bin");
        let victim = dir.path().join("victim.sha256");
        let digest = dir.path().join("manifest.sha256");
        write_file(&output, b"manifest-existing").unwrap();
        write_file(&victim, b"victim-existing").unwrap();
        symlink(&victim, &digest).unwrap();

        let error = validate_destinations(&output, &digest).unwrap_err();

        assert!(error.contains("must differ"));
        assert_unchanged(&output, b"manifest-existing");
        assert_unchanged(&victim, b"victim-existing");
        assert!(std::fs::symlink_metadata(&digest)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn distinct_destinations_are_accepted() {
        let dir = tempfile::tempdir().unwrap();
        validate_destinations(
            &dir.path().join("manifest.bin"),
            &dir.path().join("manifest.sha256"),
        )
        .unwrap();
    }
}
