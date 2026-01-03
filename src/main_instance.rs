use crate::support;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use walkdir::WalkDir;

struct ScanResult {
  path: String,
  size_mb: f64,
}

pub fn initialice_main_ui() {
  let mut user_desire_path = String::new();
  let mut results: Vec<ScanResult> = Vec::new();
  const TITLE: &str = "DiskScrutiny";

  let system = support::init(file!());

  system.main_loop(move |_, ui| {
    ui.dockspace_over_main_viewport();
    ui.window(TITLE)
      .size([500.0, 400.0], imgui::Condition::FirstUseEver)
      .build(|| {
        ui.text("Enter path to scan");
        ui.same_line();
        ui.text_disabled("(leave emtpy for current path)");
        if ui.input_text("###path", &mut user_desire_path).build() {
          dbg!(&user_desire_path);
        }
        ui.same_line();

        if ui.button("Scan now"){
          let target_path = if user_desire_path.trim().is_empty() {
            PathBuf::from(".")
          } else {
            PathBuf::from(&user_desire_path)
          };

          results = perform_scan(&target_path);
        }
        ui.separator();

        if !results.is_empty() {
          ui.text(format!("Top 10 results in {}", user_desire_path));
          ui.separator();
          for item in &results {
            ui.text(format!("{:>10.2} MB  |\t {}", item.size_mb, item.path));
          }
        } else {
          ui.text("No scan results yet");
        }

      });
  });
}

fn perform_scan(root: &Path) -> Vec<ScanResult> {
  let mut dir_sizes: HashMap<PathBuf, u64> = HashMap::new();

  for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
    if entry.file_type().is_file() {
      let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
      let mut current_path = entry.path().parent();

      while let Some(path) = current_path {
        *dir_sizes.entry(path.to_path_buf()).or_insert(0) += size;
        if path == root { break; }
        current_path = path.parent();
      }
    }
  }

  let mut sorted: Vec<(&PathBuf, &u64)> = dir_sizes.iter().collect();
  sorted.sort_by(|a, b| b.1.cmp(a.1));

  sorted.into_iter()
    .take(10)
    .map(|(path, size)| ScanResult {
      path: path.to_string_lossy().into_owned(),
      size_mb: *size as f64 / 1_000_000.0,
    })
    .collect()
}
