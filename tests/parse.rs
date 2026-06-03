//! Integration tests for the `epax parse` and `epax inspect` commands.
#![cfg(feature = "parse")]

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use assert_cmd::Command;
use tempfile::TempDir;
use zip::write::SimpleFileOptions;

fn epax() -> Command {
    Command::cargo_bin("epax").unwrap()
}

/// Helper to create a zip file with given entries (for docx, pptx, etc.)
fn create_zip_archive(path: &Path, entries: &[(&str, &str)]) {
    let file = File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    for &(name, content) in entries {
        zip.start_file(name, SimpleFileOptions::default()).unwrap();
        zip.write_all(content.as_bytes()).unwrap();
    }
    zip.finish().unwrap();
}

#[test]
fn test_parse_docx() {
    let work = TempDir::new().unwrap();
    let docx_path = work.path().join("test.docx");
    
    // Minimal word/document.xml
    let doc_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
    <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
      <w:body>
        <w:p>
          <w:r><w:t>Hello from </w:t></w:r>
          <w:r><w:t>DOCX test!</w:t></w:r>
        </w:p>
        <w:p>
          <w:r><w:t>Second paragraph.</w:t></w:r>
        </w:p>
      </w:body>
    </w:document>"#;

    create_zip_archive(&docx_path, &[("word/document.xml", doc_xml)]);

    // 1. Plain text parse
    epax()
        .arg("parse")
        .arg(&docx_path)
        .assert()
        .success()
        .stdout("Hello from DOCX test!\n\nSecond paragraph.\n");

    // 2. Markdown parse
    epax()
        .args(["parse", "--format", "md"])
        .arg(&docx_path)
        .assert()
        .success()
        .stdout("## Page 1\n\nHello from DOCX test!\n\nSecond paragraph.\n\n");

    // 3. JSON parse
    epax()
        .args(["parse", "--format", "json"])
        .arg(&docx_path)
        .assert()
        .success()
        .stdout("[\n  {\"section\":\"Page 1\",\"text\":\"Hello from DOCX test!\\n\\nSecond paragraph.\"}\n]\n");

    // 4. Output to file
    let out_txt = work.path().join("out.txt");
    epax()
        .args(["parse", "-o"])
        .arg(&out_txt)
        .arg(&docx_path)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(&out_txt).unwrap(),
        "Hello from DOCX test!\n\nSecond paragraph.\n"
    );

    // 5. Inspect command
    epax()
        .arg("inspect")
        .arg(&docx_path)
        .assert()
        .success()
        .stdout(predicates::str::contains("format  : DOCX (native Rust parser)"))
        .stdout(predicates::str::contains("sections: 1"))
        .stdout(predicates::str::contains("section 1:"));
}

#[test]
fn test_parse_pptx() {
    let work = TempDir::new().unwrap();
    let pptx_path = work.path().join("test.pptx");
    
    // Slide 1
    let slide1_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
    <p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
      <p:cSld>
        <p:spTree>
          <p:sp>
            <p:txBody>
              <a:p>
                <a:r><a:t>Slide 1 Title</a:t></a:r>
              </a:p>
              <a:p>
                <a:r><a:t>Slide 1 Subtitle</a:t></a:r>
              </a:p>
            </p:txBody>
          </p:sp>
        </p:spTree>
      </p:cSld>
    </p:sld>"#;

    // Slide 2
    let slide2_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
    <p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
      <p:cSld>
        <p:spTree>
          <p:sp>
            <p:txBody>
              <a:p>
                <a:r><a:t>Slide 2 Content</a:t></a:r>
              </a:p>
            </p:txBody>
          </p:sp>
        </p:spTree>
      </p:cSld>
    </p:sld>"#;

    create_zip_archive(
        &pptx_path,
        &[
            ("ppt/slides/slide2.xml", slide2_xml),
            ("ppt/slides/slide1.xml", slide1_xml),
        ],
    );

    // 1. Plain text parse (must sort slide1 before slide2)
    epax()
        .arg("parse")
        .arg(&pptx_path)
        .assert()
        .success()
        .stdout("Slide 1 Title\nSlide 1 Subtitle\nSlide 2 Content\n");

    // 2. Markdown parse
    epax()
        .args(["parse", "--format", "md"])
        .arg(&pptx_path)
        .assert()
        .success()
        .stdout("## Slide 1\n\nSlide 1 Title\nSlide 1 Subtitle\n\n## Slide 2\n\nSlide 2 Content\n\n");

    // 3. Inspect command
    epax()
        .arg("inspect")
        .arg(&pptx_path)
        .assert()
        .success()
        .stdout(predicates::str::contains("format  : PPTX (native Rust parser)"))
        .stdout(predicates::str::contains("slides  : 2"))
        .stdout(predicates::str::contains("slide 1:"))
        .stdout(predicates::str::contains("slide 2:"));
}

#[test]
fn test_parse_ods() {
    let work = TempDir::new().unwrap();
    let ods_path = work.path().join("test.ods");

    // ODS requires mimetype and content.xml.
    let mimetype = "application/vnd.oasis.opendocument.spreadsheet";
    let manifest_xml = r#"<?xml version="1.0" encoding="UTF-8"?><manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2"><manifest:file-entry manifest:full-path="/" manifest:version="1.2" manifest:media-type="application/vnd.oasis.opendocument.spreadsheet"/><manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/></manifest:manifest>"#;

    let content_xml = r#"<?xml version="1.0" encoding="UTF-8"?><office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" office:version="1.2"><office:body><office:spreadsheet><table:table table:name="Sheet A"><table:table-row><table:table-cell office:value-type="string"><text:p>Cell A1</text:p></table:table-cell><table:table-cell office:value-type="float" office:value="42"><text:p>42</text:p></table:table-cell></table:table-row></table:table></office:spreadsheet></office:body></office:document-content>"#;

    create_zip_archive(
        &ods_path,
        &[
            ("mimetype", mimetype),
            ("META-INF/manifest.xml", manifest_xml),
            ("content.xml", content_xml),
        ],
    );

    // Test parser on ODS via calamine (since calamine parses ODS automatically)
    epax()
        .arg("parse")
        .arg(&ods_path)
        .assert()
        .success()
        .stdout("Cell A1\t42\n");

    // Markdown parse
    epax()
        .args(["parse", "--format", "md"])
        .arg(&ods_path)
        .assert()
        .success()
        .stdout("## Sheet: Sheet A\n\nCell A1\t42\n\n");

    // Inspect command
    epax()
        .arg("inspect")
        .arg(&ods_path)
        .assert()
        .success()
        .stdout(predicates::str::contains("format  : XLSX (native Rust parser)"))
        .stdout(predicates::str::contains("sheets  : 1"))
        .stdout(predicates::str::contains("sheet 'Sheet A':"));
}
