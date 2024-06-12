#![allow(non_snake_case)]

use std::fs::{self, OpenOptions};
use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use pyo3::prelude::*;

#[cfg(target_os = "linux")]
pub fn findFileManager() -> String {
    let output = Command::new("xdg-mime")
        .args(["query", "default", "inode/directory"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_lowercase()
}

#[pyfunction]
pub fn touch(file_path: &str) {
    OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(file_path)
        .unwrap();
}

// xdgOpen opens files or folders
#[pyfunction]
#[pyo3(signature = (file_path, f_type="file", path="file"))]
pub fn xdgOpen(file_path: &str, f_type: &str, path: &str) {
    // we have a file path and we want to open it's directory.
    // highlit(select) file in file manager after opening.
    // it's help to find file easier :)
    let highlight = f_type == "folder" && path == "file";

    // for linux and bsd
    #[cfg(target_os = "linux")]
    {
        let file_manager = findFileManager();
        // check default file manager.
        // some file managers wouldn't support highlighting.
        if highlight {
            // dolphin is kde plasma's file manager
            if file_manager.contains("dolphin") {
                Command::new("dolphin")
                    .args(["--select", file_path])
                    .stderr(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stdin(Stdio::piped())
                    .status()
                    .unwrap();
            }
            // dde-file-manager is deepin's file manager
            else if file_manager.contains("dde-file-manager") {
                Command::new("dde-file-manager")
                    .args(["--show-item", file_path])
                    .stderr(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stdin(Stdio::piped())
                    .status()
                    .unwrap();
            }
            // if file manager is nautilus or nemo or pantheon-file-manager
            else if [
                "org.gnome.nautilus.desktop",
                "nemo.desktop",
                "io.elementary.files.desktop",
            ]
            .contains(&file_manager.as_str())
            {
                // nautilus is gnome's file manager.
                let file_manager = if file_manager.contains("nautilus") {
                    "nautilus"
                }
                // pantheon-files is pantheon's file manager(elementary OS).
                else if file_manager.contains("elementary") {
                    "io.elementary.files"
                }
                // nemo is cinnamon's file manager.
                else if file_manager.contains("nemo") {
                    "nemo"
                } else {
                    file_manager.as_str()
                };

                Command::new(file_manager)
                    .arg(file_path)
                    .stderr(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stdin(Stdio::piped())
                    .status()
                    .unwrap();
            } else {
                // find folder path
                let folder_path = Path::new(file_path).parent().unwrap();

                Command::new("xdg-open")
                    .arg(folder_path)
                    .stderr(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stdin(Stdio::piped())
                    .status()
                    .unwrap();
            }
        } else {
            Command::new("xdg-open")
                .arg(file_path)
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .status()
                .unwrap();
        }
    }

    // for Mac OS X
    #[cfg(target_os = "macos")]
    {
        if highlight {
            Command::new("open")
                .args(["-R", file_path])
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .status()
                .unwrap();
        } else {
            Command::new("open")
                .arg(file_path)
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .status()
                .unwrap();
        }
    }

    // for MS Windows
    #[cfg(target_os = "windows")]
    {
        const NO_WINDOW: u32 = 0x08000000;

        if highlight {
            Command::new("explorer.exe")
                .args(["/select", file_path])
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .creation_flags(NO_WINDOW)
                .status()
                .unwrap();
        } else {
            Command::new("cmd")
                .args(["/C", "start", file_path, file_path])
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .creation_flags(NO_WINDOW)
                .status()
                .unwrap();
        }
    }
}

// remove file with path of file_path
#[pyfunction]
pub fn remove(file_path: &str) -> String {
    if Path::new(file_path).is_file() {
        let result = fs::remove_file(file_path);
        match result {
            // function returns  ok, if operation was successful
            Ok(_) => "ok".to_string(),
            // function returns this, if operation was not successful
            Err(_) => "cant".to_string(),
        }
    } else {
        // function returns this , if file is not existed
        "no".to_string()
    }
}

// removeDir removes folder : folder_path
#[pyfunction]
pub fn removeDir(folder_path: &str) -> String {
    // check folder_path existence
    if Path::new(folder_path).is_dir() {
        // remove folder
        let result = fs::remove_dir_all(folder_path);
        match result {
            Ok(_) => "ok".to_string(),
            // return 'cant' if removing was not successful
            Err(_) => "cant".to_string(),
        }
    } else {
        // return 'no' if file didn't existed
        "no".to_string()
    }
}

// make directory
#[pyfunction]
#[pyo3(signature = (folder_path, hidden=false))]
pub fn makeDirs(folder_path: &str, hidden: bool) -> String {
    if hidden {
        #[cfg(target_os = "windows")]
        {
            // create hidden attribute directory.

            fs::create_dir_all(folder_path).unwrap();

            const NO_WINDOW: u32 = 0x08000000;
            Command::new("attrib")
                .args(["+h", folder_path])
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .creation_flags(NO_WINDOW)
                .status()
                .unwrap();
        }

        #[cfg(not(target_os = "windows"))]
        {
            // In linux and bsd a dot character must be added in the start of the directory's name
            let dir_name = Path::new(folder_path).file_name().unwrap();
            let parent_path = Path::new(folder_path).parent().unwrap();
            let folder_path =
                Path::new(parent_path).join(".".to_owned() + dir_name.to_str().unwrap());

            fs::create_dir_all(folder_path).unwrap();
        }
    } else {
        fs::create_dir_all(folder_path).unwrap();
    }

    folder_path.to_string()
}

// move downloaded file to another destination.
#[pyfunction]
#[pyo3(signature = (old_file_path, new_path, new_path_type="folder"))]
pub fn moveFile(old_file_path: &str, new_path: &str, new_path_type: &str) -> bool {
    // new_path_type can be file or folder
    // if it's folder so we have folder path
    // else we have new file path that includes file name
    if Path::new(old_file_path).is_file() {
        let check_path = if new_path_type == "folder" {
            // check availability of directory
            Path::new(new_path).is_dir()
        } else {
            true
        };
        if check_path {
            // move file to new_path
            let result = fs::rename(old_file_path, new_path);
            result.is_ok()
        } else {
            false
        }
    } else {
        false
    }
}
