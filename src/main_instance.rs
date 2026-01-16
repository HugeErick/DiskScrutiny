use std::path::{Path, PathBuf};
use ntfs::NtfsAttributeType;
use rfd::FileDialog;
use std::collections::HashMap;
use walkdir::WalkDir;

#[cfg(windows)]
use std::fs::File;
#[cfg(windows)]
use ntfs::Ntfs;

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

        if ui.button("Choose folder") {
          if let Some(path) = FileDialog::new().pick_folder() {
            results = perform_scan(&path);
            selected_path = Some(path);
          }
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
    use ntfs::NtfsFileFlags;

    let drive_str = root.to_str().ok_or("invalid path")?;
    if drive_str.len() < 2 { return Err("Invalid drive".into()); }
    let disk_name = format!("\\\\.\\{}", &drive_str[0..2]); 

    let mut disk = File::open(disk_name)?;
    let ntfs = Ntfs::new(&mut disk)?;

    let mft_file = ntfs.file(&mut disk, 0)?;
    let mut mft_entry_count = 0u64;

    let mut mft_attrs = mft_file.attributes();
    while let Some(Ok(attr_item)) = mft_attrs.next(&mut disk) {
      let attr = attr_item.to_attribute()?;
      if attr.ty()? == NtfsAttributeType::Data {
        mft_entry_count = attr.value_length() / (ntfs.file_record_size() as u64);
        break;
      }
    }

    let mut dir_sizes: HashMap<u64, u64> = HashMap::new();
    let mut id_to_name: HashMap<u64, String> = HashMap::new();

    for record_num in 0..mft_entry_count {
      let Ok(file) = ntfs.file(&mut disk, record_num) else { continue };

      // Check if file is in use using the contains method from bitflags
      let flags = file.flags();
      if !flags.contains(NtfsFileFlags::IN_USE) {
        continue;
      }

      if let Some(Ok(name_res)) = file.name(&mut disk, None, None) {
        let name_str = name_res.name().to_string().unwrap_or_default();
        id_to_name.insert(record_num, name_str);

        let parent_id = name_res.parent_directory_reference().file_record_number();

        let mut file_attrs = file.attributes();
        while let Some(Ok(attr_item)) = file_attrs.next(&mut disk) {
          let attr = attr_item.to_attribute()?;
          if attr.ty()? == NtfsAttributeType::Data {
            let size = attr.value_length();
            *dir_sizes.entry(parent_id).or_insert(0) += size;
            break; 
          }
        }
      }
    }

    let mut results: Vec<ScanResult> = dir_sizes
      .into_iter()
      .map(|(id, size)| {
        let name = id_to_name.get(&id)
          .cloned()
          .unwrap_or_else(|| format!("dir_{}", id));

        ScanResult {
          path: name,
          size_mb: size as f64 / 1_048_576.0,
        }
      })
    .collect();

    results.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap_or(std::cmp::Ordering::Equal));
    Ok(results.into_iter().take(10).collect())
  }
}
