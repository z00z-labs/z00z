//! Generate the sole bounded application action-basis manifest.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use z00z_core::actions::LifecycleEffectV1;
use z00z_crypto::hash::sha256_256_simple;
use z00z_utils::codec::{AppWireCodec, AppWireEnvelope, AppWireField};

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
        AppWireField::new(3, sha256_256_simple(ACTION_DESCRIPTOR_SOURCE).to_vec()),
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
        AppWireField::new(4, sha256_256_simple(&basis_preimage).to_vec()),
    );
    let bytes = AppWireCodec
        .encode(&AppWireEnvelope::v1(fields))
        .map_err(|error| error.to_string())?;
    let digest = sha256_256_simple(&bytes);

    write_atomic(&output, &bytes)?;
    write_atomic(&digest_output, format!("{}\n", hex(&digest)).as_bytes())?;
    println!(
        "generated {} effects: {} ({})",
        LifecycleEffectV1::ATOMIC_BASIS.len(),
        output.display(),
        hex(&digest)
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

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("output has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    fs::rename(&temporary, path).map_err(|error| error.to_string())
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[(byte >> 4) as usize] as char);
        output.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    output
}
