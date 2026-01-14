use std::path::{Path, PathBuf};
use rfd::FileDialog;
use std::collections::HashMap;
use walkdir::WalkDir;

#[cfg(windows)]
use std::fs::File;
#[cfg(windows)]
use ntfs::Ntfs;

// This must be gated or removed to compile on Linux
#[cfg(windows)]
use std::os::windows::fs::FileExt;

use crate::support;
use crate::utils::os_identity::TargetOS;

struct ScanResult {
  path: String,
  size_mb: f64,
}

pub fn initialice_main_ui() {
  let mut selected_path: Option<PathBuf> = None;
  let mut results: Vec<ScanResult> = Vec::new();
  const TITLE: &str = "DiskScrutiny";

  let system = support::init(file!());

  system.main_loop(move |_, ui| {
    ui.dockspace_over_main_viewport();
    ui.window(TITLE) 
      .size([500.0, 400.0], imgui::Condition::FirstUseEver) // Fixed case: Condition
      .build(|| {
        ui.text("Enter path to scan");
        ui.same_line();

        if ui.button("Choose folder")
        && let Some(path) = FileDialog::new().pick_folder() {
          results = perform_scan(&path);
          selected_path = Some(path);

        }
        ui.separator();

        if !results.is_empty() {
          let path_name = selected_path.as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "current directory".to_string());

          ui.text(format!("Top 10 results in {}", path_name));
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
  let os = crate::utils::os_identity::identification();

  match os {
    TargetOS::Windows => {
      scan_mft(root).unwrap_or_else(|_| scan_walkdir(root))
    }
    _ => scan_walkdir(root),
  }
}

fn scan_walkdir(root: &Path) -> Vec<ScanResult> { 
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

fn scan_mft(root: &Path) -> Result<Vec<ScanResult>, Box<dyn std::error::Error>> {
  #[cfg(unix)]
  {
    let _ = root; 
    Err("MFT scanning only supported on Windows".into())
  }

  #[cfg(windows)]
  {
    let drive_str = root.to_str().ok_or("invalid path")?;
    if drive_str.len() < 2 { return Err("Invalid drive".into()); }
    let disk_name = format!("\\\\.\\{}", &drive_str[0..2]); 

    let mut disk = File::open(disk_name)?;
    let ntfs = Ntfs::new(&mut disk)?;

    let mut dir_sizes: HashMap<u64, u64> = HashMap::new();
    let mut id_to_name: HashMap<u64, String> = HashMap::new();

    for entry in ntfs.all_files(&mut disk) {
      let Ok(file) = entry else { continue };
      let file_id = file.file_record_number();

      if let Some(Ok(name)) = file.name(&mut disk) {
        id_to_name.insert(file_id, name.name().to_string());
      }

      if let Some(Ok(data)) = file.data_attribute(&mut disk) {
        let size = data.value_length();
        if let Some(parent_ref) = file.parent_directory_reference() {
          let parent_id = parent_ref.file_record_number();
          *dir_sizes.entry(parent_id).or_insert(0) += size;
        }
      }
    }

    let mut results: Vec<ScanResult> = dir_sizes
      .into_iter()
      .map(|(id, size)| {
        let name = id_to_name.get(&id)
          .map(|s| s.to_string())
          .unwrap_or_else(|| format!("directory id {}", id));

        ScanResult {
          path: name,
          size_mb: size as f64 / 1_000_000.0,
        }
      })
      .collect();

    results.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap_or(std::cmp::Ordering::Equal));
    Ok(results.into_iter().take(10).collect())
  }
}
