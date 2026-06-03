//! End-to-end tests: drive the compiled `epax` binary through compress →
//! extract round trips for every writable format, plus RAR extract-only.

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

/// Build a small source tree under `dir`: two files in nested dirs.
fn make_source(dir: &Path) {
    fs::create_dir_all(dir.join("sub")).unwrap();
    fs::write(dir.join("hello.txt"), b"hello epax\n").unwrap();
    fs::write(dir.join("sub/data.bin"), vec![0u8, 1, 2, 3, 255, 128, 7]).unwrap();
}

fn epax() -> Command {
    Command::cargo_bin("epax").unwrap()
}

/// Compress `src_dir` into an archive named `archive_name`, extract it into a
/// fresh dir, and assert the two known files survive byte-for-byte.
fn roundtrip(archive_name: &str) {
    let work = TempDir::new().unwrap();
    let src = work.path().join("src");
    make_source(&src);

    let archive = work.path().join(archive_name);
    epax()
        .arg("compress")
        .arg(&archive)
        .arg(&src)
        .assert()
        .success();
    assert!(archive.exists(), "archive {archive_name} was not created");

    let out = work.path().join("out");
    epax()
        .args(["extract", "-o"])
        .arg(&out)
        .arg(&archive)
        .assert()
        .success();

    // Entries are stored rooted at the input's name ("src/...").
    let hello = out.join("src/hello.txt");
    let data = out.join("src/sub/data.bin");
    assert_eq!(
        fs::read(&hello).unwrap(),
        b"hello epax\n",
        "{archive_name}: hello.txt mismatch"
    );
    assert_eq!(
        fs::read(&data).unwrap(),
        vec![0u8, 1, 2, 3, 255, 128, 7],
        "{archive_name}: data.bin mismatch"
    );
}

#[test]
fn roundtrip_zip() {
    roundtrip("out.zip");
}

#[test]
fn roundtrip_7z() {
    roundtrip("out.7z");
}

#[test]
fn roundtrip_tar() {
    roundtrip("out.tar");
}

#[test]
fn roundtrip_tar_gz() {
    roundtrip("out.tar.gz");
}

#[test]
fn roundtrip_tgz() {
    roundtrip("out.tgz");
}

#[test]
fn roundtrip_tar_bz2() {
    roundtrip("out.tar.bz2");
}

#[test]
fn roundtrip_tar_zst() {
    roundtrip("out.tar.zst");
}

/// A single file compressed to a bare `.gz` must decompress back to that file
/// (no tar container, original name restored).
#[test]
fn roundtrip_bare_gz_single_file() {
    let work = TempDir::new().unwrap();
    let file = work.path().join("notes.txt");
    fs::write(&file, b"bare stream content").unwrap();

    let archive = work.path().join("notes.txt.gz");
    epax()
        .arg("compress")
        .arg(&archive)
        .arg(&file)
        .assert()
        .success();

    let out = work.path().join("out");
    epax()
        .args(["extract", "-o"])
        .arg(&out)
        .arg(&archive)
        .assert()
        .success();

    assert_eq!(
        fs::read(out.join("notes.txt")).unwrap(),
        b"bare stream content"
    );
}

/// `list` should enumerate entries without extracting.
#[test]
fn list_zip_shows_entries() {
    let work = TempDir::new().unwrap();
    let src = work.path().join("src");
    make_source(&src);
    let archive = work.path().join("out.zip");
    epax()
        .arg("compress")
        .arg(&archive)
        .arg(&src)
        .assert()
        .success();

    epax()
        .arg("list")
        .arg(&archive)
        .assert()
        .success()
        .stdout(predicates::str::contains("hello.txt"))
        .stdout(predicates::str::contains("data.bin"));
}

/// `--format` overrides extension detection: a zip archive with a non-standard
/// name is still listed correctly when the format is forced.
#[test]
fn format_override_lists_misnamed_archive() {
    let work = TempDir::new().unwrap();
    let src = work.path().join("src");
    make_source(&src);
    // Create a real zip but give it a name epax could not auto-detect.
    let archive = work.path().join("bundle.dat");
    epax()
        .args(["compress", "--format", "zip"])
        .arg(&archive)
        .arg(&src)
        .assert()
        .success();

    epax()
        .args(["list", "--format", "zip"])
        .arg(&archive)
        .assert()
        .success()
        .stdout(predicates::str::contains("hello.txt"));
}

/// RAR can be extracted from the committed fixture.
#[cfg(feature = "rar")]
#[test]
fn rar_extract_fixture() {
    let work = TempDir::new().unwrap();
    let out = work.path().join("out");
    epax()
        .args(["extract", "-o"])
        .arg(&out)
        .arg("tests/fixtures/sample.rar")
        .assert()
        .success();
    // The fixture contains a single entry named "VERSION".
    assert!(
        out.join("VERSION").exists(),
        "expected extracted VERSION file"
    );
}

/// Creating a RAR must fail clearly with exit code 2.
#[test]
fn rar_compress_is_rejected() {
    let work = TempDir::new().unwrap();
    let file = work.path().join("a.txt");
    fs::write(&file, b"x").unwrap();
    let archive = work.path().join("out.rar");

    epax()
        .arg("compress")
        .arg(&archive)
        .arg(&file)
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("not supported"));
}

/// An unknown extension is a clear error.
#[test]
fn unknown_extension_errors() {
    let work = TempDir::new().unwrap();
    let file = work.path().join("a.txt");
    fs::write(&file, b"x").unwrap();

    epax()
        .arg("compress")
        .arg(work.path().join("out.weird"))
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicates::str::contains("could not determine format"));
}
