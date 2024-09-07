use std::fs::File;

use camino::Utf8PathBuf;
use tar::Archive;
use tempdir::TempDir;

use cargo_skeleton::{build, BuildOptions};

#[test]
fn skeleton() {
    let tmp_dir = TempDir::new("cargo-skeleton").expect("creating temp dir");
    let out_path: Utf8PathBuf = tmp_dir.path().join("skeleton.tar").try_into().expect("converting path to UTF-8");

    let opts = BuildOptions {
        out_path: Some(out_path.clone()),
        ..Default::default()
    };

    build(opts).expect("building skeleton");

    let file = File::open(&out_path).expect("opening out file");
    let mut ar = Archive::new(file);

    let entries = ar.entries().unwrap();

    for file in entries {
        let file = file.unwrap();

        let path = file.header()
            .path().unwrap().into_owned()
            .to_str().unwrap().to_string();

        match path.as_str() {
            "Cargo.toml" => {

            },
            "src/lib.rs" => {

            },
            "src/main.rs" | "tests/self_test.rs" => {

            },
            _ => panic!("Unexpected path {}", path),
        }
    }

    // let paths: Vec<String> = entries
    //     .find(|f| f.unwrap()
    //         .header().path().unwrap().into_owned()
    //         .to_string_lossy()
    //         .into_owned()
    //     )
    //     .collect();

    // assert!(paths.iter().any(|p| p == "Cargo.toml"));
    // assert!(paths.iter().any(|p| p == "src/lib.rs"));
    // assert!(paths.iter().any(|p| p == "src/main.rs"));

    // TODO: assert header attributes too
}
