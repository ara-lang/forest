use std::env;

use ara_forest::config::Config;
use ara_forest::error::Error;
use ara_forest::source::SourceFilesCollector;

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn test_collecting_files_in_project_a() {
    let root = format!("{MANIFEST_DIR}/tests/examples/project-a");
    let config = Config::new(&root).with_source("src").with_definitions(vec![
        format!("vendor/std-bar/definitions"),
        format!("vendor/std-foo/definitions"),
    ]);
    let files = SourceFilesCollector::new(&config).collect().unwrap();

    assert_eq!(files.len(), 6);

    let source = format!("{root}/src");
    assert!(files.contains(&format!("{source}/foo.ara").into()));
    assert!(files.contains(&format!("{source}/Bar/bar.ara").into()));
    assert!(files.contains(&format!("{source}/Foo/Bar/Baz/baz.ara").into()));
    assert!(files.contains(&format!("{source}/Foo/Bar/bar.ara").into()));

    let vendor = format!("{root}/vendor");
    assert!(files.contains(&format!("{vendor}/std-foo/definitions/std-foo.d.ara").into()));
    assert!(files.contains(&format!("{vendor}/std-bar/definitions/std-bar.d.ara").into()));
}

#[test]
fn test_trying_to_collect_files_in_a_fake_directory() {
    let root = format!("{MANIFEST_DIR}/tests/examples/project-fake");

    let config = Config::new(root).with_source("src");
    let result = SourceFilesCollector::new(&config).collect();

    assert!(
        matches!(result, Err(Error::InvalidPath(_))),
        "Expected an InvalidSource error, but got something else",
    );
}

#[test]
fn test_trying_to_collect_files_in_a_invalid_path() {
    let root = format!("{MANIFEST_DIR}/tests/examples/project-a");

    let config = Config::new(root).with_source("src/foo.ara");
    let result = SourceFilesCollector::new(&config).collect();

    assert!(
        matches!(result, Err(Error::InvalidPath(_))),
        "Expected an InvalidSource error, but got something else",
    );
}
