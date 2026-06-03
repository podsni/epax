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

/// With an unrecognized extension and an explicit second input the first arg
/// is treated as an extra input (auto-output mode → stem + .zip), so the error
/// is about the non-existent path, not an unknown format.
#[test]
fn unknown_extension_becomes_auto_output_mode() {
    let work = TempDir::new().unwrap();
    let file = work.path().join("a.txt");
    fs::write(&file, b"x").unwrap();

    // "out.weird" has no archive extension → treated as input, fails because it
    // does not exist on disk.
    epax()
        .arg("compress")
        .arg(work.path().join("out.weird"))
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicates::str::contains("does not exist"));
}

/// `epax compress aku.md` — no archive extension on the first arg → auto-generates
/// `aku.zip` in the same directory and treats the original arg as the input.
#[test]
fn compress_auto_output_zip() {
    let work = TempDir::new().unwrap();
    let file = work.path().join("notes.md");
    fs::write(&file, b"auto output test").unwrap();

    // Run from work dir so relative output lands there.
    epax()
        .current_dir(work.path())
        .arg("compress")
        .arg("notes.md")
        .assert()
        .success()
        .stdout(predicates::str::contains("notes.zip"));

    assert!(work.path().join("notes.zip").exists());
}

/// `epax compress doc.pdf --format 7z` → auto-output uses the forced format.
#[test]
fn compress_auto_output_with_format_override() {
    let work = TempDir::new().unwrap();
    let file = work.path().join("report.pdf");
    fs::write(&file, b"pdf content").unwrap();

    epax()
        .current_dir(work.path())
        .args(["compress", "report.pdf", "--format", "7z"])
        .assert()
        .success();

    // With --format given, the auto-output stem is still "report" but the
    // format is 7z — the output written is report.7z.
    // (The exact output name depends on the auto-output logic; we just verify
    //  that a 7z archive was created somewhere in the work dir.)
    let found = fs::read_dir(work.path())
        .unwrap()
        .any(|e| e.unwrap().file_name().to_string_lossy().ends_with(".7z"));
    assert!(found, "expected a .7z file in work dir");
}

/// Extract a zip renamed to `.dat` — magic-byte detection should identify it.
#[test]
fn extract_magic_detection_zip_as_dat() {
    let work = TempDir::new().unwrap();
    let src = work.path().join("src");
    make_source(&src);

    // Create a real zip but store it with an unrecognized extension.
    let zip_path = work.path().join("bundle.zip");
    epax()
        .arg("compress")
        .arg(&zip_path)
        .arg(&src)
        .assert()
        .success();

    let dat_path = work.path().join("bundle.dat");
    fs::rename(&zip_path, &dat_path).unwrap();

    let out = work.path().join("out");
    epax()
        .args(["extract", "-o"])
        .arg(&out)
        .arg(&dat_path)
        .assert()
        .success();

    assert!(out.join("src/hello.txt").exists());
}

/// `epax e` alias for extract works.
#[test]
fn alias_e_for_extract() {
    let work = TempDir::new().unwrap();
    let src = work.path().join("src");
    make_source(&src);

    let archive = work.path().join("out.zip");
    epax().arg("compress").arg(&archive).arg(&src).assert().success();

    let out = work.path().join("out");
    epax()
        .arg("e")
        .arg(&archive)
        .args(["-o"])
        .arg(&out)
        .assert()
        .success();

    assert!(out.join("src/hello.txt").exists());
}
