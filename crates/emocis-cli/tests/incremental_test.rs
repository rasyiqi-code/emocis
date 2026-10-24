use emocis_cli::common::index_directory_ext;
use std::fs;
use std::path::PathBuf;

struct TempProject(PathBuf);

impl TempProject {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "emocis-incr-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let path = root.join(name);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, content: &str) {
        let full_path = self.0.join(name);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(full_path, content).unwrap();
    }

    fn remove(&self, name: &str) {
        let full_path = self.0.join(name);
        let _ = fs::remove_file(full_path);
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn test_incremental_indexing_cache_and_invalidation() {
    let proj = TempProject::new("test_incr_app");
    proj.write("src/a.ts", "export function funcA() { return 1; }\n// @decision Use A");
    proj.write("src/b.ts", "export function funcB() { return 2; }\n// @warning Watch B");

    // 1. Initial indexing: both files must be indexed, 0 cached
    let stats1 = index_directory_ext(&proj.0, "test_incr_app", false).unwrap();
    assert_eq!(stats1.total_files, 2);
    assert_eq!(stats1.indexed_files, 2);
    assert_eq!(stats1.cached_files, 0);
    assert_eq!(stats1.total_knowledge, 2);

    // 2. Second indexing without changes: both files must be cached (skipped)
    let stats2 = index_directory_ext(&proj.0, "test_incr_app", false).unwrap();
    assert_eq!(stats2.total_files, 2);
    assert_eq!(stats2.indexed_files, 0);
    assert_eq!(stats2.cached_files, 2);
    assert_eq!(stats2.total_knowledge, 2);

    // 3. Modify only src/a.ts: src/b.ts cached, src/a.ts re-indexed
    proj.write(
        "src/a.ts",
        "export function funcA() { return 100; }\n// @decision Updated Decision for A\n// @rule Rule for A",
    );
    let stats3 = index_directory_ext(&proj.0, "test_incr_app", false).unwrap();
    assert_eq!(stats3.total_files, 2);
    assert_eq!(stats3.indexed_files, 1);
    assert_eq!(stats3.cached_files, 1);
    // Knowledge count should now be 3 (2 from a.ts + 1 from b.ts)
    assert_eq!(stats3.total_knowledge, 3);

    // 4. Delete src/b.ts and add src/c.ts
    proj.remove("src/b.ts");
    proj.write("src/c.ts", "export function funcC() { return 3; }");
    let stats4 = index_directory_ext(&proj.0, "test_incr_app", false).unwrap();
    assert_eq!(stats4.total_files, 2);
    assert_eq!(stats4.indexed_files, 1); // c.ts is new
    assert_eq!(stats4.cached_files, 1);  // a.ts is cached
    // Total knowledge should now be 2 (b.ts deleted, its knowledge removed)
    assert_eq!(stats4.total_knowledge, 2);

    // 5. Force full re-index: both files re-indexed, 0 cached
    let stats5 = index_directory_ext(&proj.0, "test_incr_app", true).unwrap();
    assert_eq!(stats5.total_files, 2);
    assert_eq!(stats5.indexed_files, 2);
    assert_eq!(stats5.cached_files, 0);
    assert_eq!(stats5.total_knowledge, 2);
}
