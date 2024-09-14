use std::fs::File;

use camino::Utf8PathBuf;
use tar::Archive;
use tempdir::TempDir;

use cargo_skeleton::{create_skeleton, CreateOptions};

#[test]
fn skeleton() {
    let tmp_dir = TempDir::new("cargo-skeleton").expect("creating temp dir");
    let out_path: Utf8PathBuf = tmp_dir.path().join("skeleton.tar").try_into().expect("converting path to UTF-8");

    let opts = CreateOptions {
        out_path: Some(out_path.clone()),
        ..Default::default()
    };

    create_skeleton(opts).expect("building skeleton");

    let file = File::open(&out_path).expect("opening out file");
    let mut ar = Archive::new(file);

    let entries = ar.entries().unwrap();

    for file in entries {
        let file = file.unwrap();

        let _path = file.header()
            .path().unwrap().into_owned()
            .to_str().unwrap().to_string();

        // TODO: assertions
    }
}
