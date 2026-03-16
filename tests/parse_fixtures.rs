//! Integration test that discovers all files in `tests/data/` and runs a parse + insta
//! snapshot test for each. Requires `alloc` and `serde` features.
//!
//! Files whose name (stem) starts with `_` are treated as ignored tests.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args = libtest_mimic::Arguments::from_args();
    let tests = discover_tests();
    libtest_mimic::run(&args, tests).exit();
}

fn discover_tests() -> Vec<libtest_mimic::Trial> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let data_dir = PathBuf::from(&manifest_dir).join("tests").join("data");

    let mut entries: Vec<_> = fs::read_dir(&data_dir)
        .unwrap_or_else(|e| panic!("failed to read tests/data: {}", e))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    entries
        .into_iter()
        .filter(|e| e.metadata().unwrap().is_file())
        .map(|entry| TestCase::from_path(entry.path()))
        .map(|test| test.into_trial())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TestCase {
    path: PathBuf,
    name: String,
    ignored: bool,
}

impl TestCase {
    fn from_path(path: PathBuf) -> Self {
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap();

        let (name, ignored) = match name.strip_prefix("_") {
            Some(name) => (name, true),
            None => (name, false),
        };
        let path = path.canonicalize().unwrap();

        TestCase {
            path,
            name: name.to_string(),
            ignored,
        }
    }

    fn into_trial(self) -> libtest_mimic::Trial {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "alloc", feature = "serde"))] {
                const ENABLED: bool = true;
                fn run_parse_test(
                    case: &TestCase,
                ) -> Result<(), libtest_mimic::Failed> {
                    let content = std::fs::read_to_string(&case.path)
                        .map_err(|e| libtest_mimic::Failed::from(e.to_string()))?;
                    let mut program = gcode::parse(&content)
                        .map_err(|d| libtest_mimic::Failed::from(format!("{:?}", d)))?;

                    // Note: to avoid saving megabytes of snapshots, we only
                    // take the first hundred blocks or so.
                    program.blocks.truncate(100);

                    insta::with_settings!({ snapshot_suffix => &case.name }, {
                        insta::assert_debug_snapshot!(program);
                    });

                    Ok(())
                }
            } else {
                const ENABLED: bool = false;
                fn run_parse_test(
                    _case: &TestCase,
                ) -> Result<(), libtest_mimic::Failed> {
                    Err(libtest_mimic::Failed::from("not enabled"))
                }
            }
        }

        let name = self.name.clone();
        let ignored = self.ignored;

        libtest_mimic::Trial::test(name, move || run_parse_test(&self))
            .with_ignored_flag(ignored || !ENABLED)
    }
}
