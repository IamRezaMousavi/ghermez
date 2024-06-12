#![allow(non_snake_case)]

use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread, time,
};

#[cfg(target_os = "windows")]
use std::env;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use chrono::Local;
use log::{error, info};
use once_cell::sync::Lazy;
use pyo3::prelude::*;
use serde_json::{from_value, to_value, Map};
use tokio::{runtime::Runtime, sync::RwLock};

use aria2_ws::{Client, TaskOptions};

use crate::{
    response::{CustomStatus, ValuesToString as _},
    useful_tools::{humanReadableSize, round},
};

static SERVER_URL: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new(String::new()));

// start aria2 with RPC
#[pyfunction]
#[pyo3(signature = (port, _aria2_path=None))]
pub fn startAria(port: u16, _aria2_path: Option<String>) -> Option<String> {
    Runtime::new().unwrap().handle().block_on(async {
        let mut tmp = SERVER_URL.write().await;
        *tmp = format!("ws://127.0.0.1:{port}/jsonrpc");
    });

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let _child = match Command::new("aria2c")
        .arg("--no-conf")
        .arg("--enable-rpc")
        .arg(format!("--rpc-listen-port={}", port))
        .arg("--rpc-allow-origin-all")
        .arg("--quiet=true")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
    {
        Err(why) => panic!("couldn't spawn aria2c: {:?}", why),
        Ok(child) => child,
    };

    #[cfg(target_os = "windows")]
    {
        let aria2d = if _aria2_path
            .clone()
            .is_some_and(|x| !x.is_empty() && Path::new(&x).is_file())
        {
            _aria2_path.as_ref().unwrap().to_string()
        } else {
            let aria2 = env::current_dir().unwrap().join("aria2c.exe");
            aria2.to_str().unwrap().to_string()
        };

        if !Path::new(&aria2d).exists() {
            error!("Aria2 does not exist in the current path!");
            return None;
        }

        // NO_WINDOW option avoids opening additional CMD window in MS Windows.
        const NO_WINDOW: u32 = 0x08000000;

        let _child = match Command::new(aria2d)
            .arg("--no-conf")
            .arg("--enable-rpc")
            .arg(format!("--rpc-listen-port={}", port))
            .arg("--rpc-allow-origin-all")
            .arg("--quiet=true")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .creation_flags(NO_WINDOW)
            .spawn()
        {
            Err(why) => panic!("couldn't spawn aria2c: {:?}", why),
            Ok(child) => child,
        };
    }

    thread::sleep(time::Duration::from_secs(2));

    // check that starting is successful or not!
    let answer = aria2Version();
    Some(answer)
}

// check aria2 release version . Ghermez uses this function to
// check that aria2 RPC connection is available or not.
#[pyfunction]
pub fn aria2Version() -> String {
    let version = Runtime::new().unwrap().handle().block_on(async {
        let server_url = SERVER_URL.read().await;
        match Client::connect(&server_url, None).await {
            Ok(client) => client.get_version().await,
            Err(e) => Err(e),
        }
    });

    match version {
        Ok(v) => v.version,
        Err(_) => {
            // write ERROR messages in terminal and log
            error!("Aria2 didn't respond!");
            "did not respond".to_string()
        }
    }
}

fn _download_aria(url: &str) -> String {
    let gid = Runtime::new().unwrap().handle().block_on(async {
        let server_url = SERVER_URL.read().await;
        let client = Client::connect(&server_url, None).await.unwrap();
        let options = TaskOptions::default();

        client
            .add_uri(vec![url.to_string()], Some(options.clone()), None, None)
            .await
            .unwrap()
    });
    gid
}

type GidList = Vec<String>;
type DownloadStatusList = Vec<HashMap<String, Option<String>>>;

// this function returns list of download information
#[pyfunction]
pub fn tellActive() -> (Option<GidList>, Option<DownloadStatusList>) {
    let args = vec![
        "gid".to_string(),
        "status".to_string(),
        "connections".to_string(),
        "errorCode".to_string(),
        "errorMessage".to_string(),
        "downloadSpeed".to_string(),
        "dir".to_string(),
        "totalLength".to_string(),
        "completedLength".to_string(),
        "files".to_string(),
    ];
    // get download information from aria2
    let downloads_status_result = Runtime::new().unwrap().handle().block_on(async {
        let server_url = SERVER_URL.read().await;
        match Client::connect(&server_url, None).await {
            Ok(client) => client.custom_tell_active(Some(args)).await,
            Err(e) => Err(e),
        }
    });

    let downloads_status: Vec<CustomStatus> = match downloads_status_result {
        Ok(downloads_status) => from_value(to_value(downloads_status).unwrap()).unwrap(),
        Err(_) => return (None, None),
    };

    let mut download_status_list = vec![];
    let mut gid_list = vec![];

    // convert download information in desired format.
    for download_dict in downloads_status {
        let converted_info_dict = convertDownloadInformation(download_dict.clone());

        // add gid to gid_list
        gid_list.push(download_dict.gid);

        // add converted information to download_status_list
        download_status_list.push(converted_info_dict);
    }

    (Some(gid_list), Some(download_status_list))
}

fn _tellStatus(gid: &str) -> Map<String, serde_json::Value> {
    let args = vec![
        "status".to_string(),
        "connections".to_string(),
        "errorCode".to_string(),
        "errorMessage".to_string(),
        "downloadSpeed".to_string(),
        "connections".to_string(),
        "dir".to_string(),
        "totalLength".to_string(),
        "completedLength".to_string(),
        "files".to_string(),
    ];
    let status = Runtime::new().unwrap().handle().block_on(async {
        let server_url = SERVER_URL.read().await;
        let client = Client::connect(&server_url, None).await.unwrap();
        client.custom_tell_status(gid, Some(args)).await.unwrap()
    });

    status
}

// this function converts download information that received from aria2 in desired format.
// input format must be a dictionary.
fn convertDownloadInformation(download_status: CustomStatus) -> HashMap<String, Option<String>> {
    // find file_name
    let file_status = download_status.files;
    // file_status contains name of download file and link of download file
    let path = &file_status[0].path;
    let file_name = if !path.is_empty() {
        Path::new(&path)
            .file_name()
            .map(|parent| parent.to_str().unwrap().to_string())
    } else {
        None
    };

    let link = Some(file_status[0].uris[0].uri.to_owned());

    // find file_size
    let file_size = download_status.total_length;
    // find downloaded size
    let downloaded = download_status.completed_length;

    let percent_str;
    let size_str;
    let downloaded_str;
    // convert file_size and downloaded_size to KiB and MiB and GiB
    if file_size != 0 {
        // find download percent from file_size and downloaded_size
        let percent = downloaded as f32 * 100.0 / file_size as f32;

        // converting file_size to KiB or MiB or GiB
        size_str = Some(humanReadableSize(file_size as f32, "file_size"));
        downloaded_str = Some(humanReadableSize(downloaded as f32, "file_size"));

        percent_str = Some(format!("{percent}%"));
    } else {
        size_str = None;
        downloaded_str = None;
        percent_str = None;
    }

    // find download_speed
    let download_speed = download_status.download_speed;

    // convert download_speed to desired units.
    // and find estimate_time_left
    let mut estimate_time_left_str;
    let download_speed_str;
    if file_size != 0 && download_speed != 0 {
        let mut estimate_time_left = (file_size - downloaded) as f32 / download_speed as f32;

        // converting file_size to KiB or MiB or GiB
        download_speed_str = Some(humanReadableSize(download_speed as f32, "speed") + "/s");

        let mut eta = String::new();
        if estimate_time_left >= 3600.0 {
            eta += &(format!("{}", estimate_time_left / 3600.0) + "h");
            estimate_time_left %= 3600.0;
            eta += &(format!("{}", estimate_time_left / 60.0) + "m");
            estimate_time_left %= 60.0;
            eta += &(format!("{}", estimate_time_left) + "s");
        } else if estimate_time_left >= 60.0 {
            eta += &(format!("{}", estimate_time_left / 60.0) + "m");
            estimate_time_left %= 60.0;
            eta += &(format!("{}", estimate_time_left) + "s");
        } else {
            eta += &(format!("{}", estimate_time_left) + "s");
        }
        estimate_time_left_str = Some(eta);
    } else {
        download_speed_str = Some("0".to_string());
        estimate_time_left_str = None;
    }

    // find number of connections
    let connections_str = Some(download_status.connections.to_string());

    // find status of download
    let mut status_str = Some(download_status.status.to_string());

    // rename active status to downloading
    if status_str.as_ref().is_some_and(|s| s == "active") {
        status_str = Some("downloading".to_string());
    }
    // rename removed status to stopped
    else if status_str.as_ref().is_some_and(|s| s == "removed") {
        status_str = Some("stopped".to_string());
    } else if status_str.as_ref().is_some_and(|s| s == "None") {
        status_str = None;
    }
    // set 0 second for estimate_time_left_str if download is completed.
    else if status_str.as_ref().is_some_and(|s| s == "complete") {
        estimate_time_left_str = Some("0s".to_string());
    }

    HashMap::from([
        ("gid".to_string(), Some(download_status.gid)),
        ("file_name".to_string(), file_name),
        ("status".to_string(), status_str),
        ("size".to_string(), size_str),
        ("downloaded_size".to_string(), downloaded_str),
        ("percent".to_string(), percent_str),
        ("connections".to_string(), connections_str),
        ("rate".to_string(), download_speed_str),
        ("estimate_time_left".to_string(), estimate_time_left_str),
        ("link".to_string(), link),
    ])
}

// this function returns folder of download according to file extension
#[pyfunction]
pub fn findDownloadPath(file_name: &str, download_path: PathBuf, subfolder: &str) -> PathBuf {
    if subfolder != "yes" {
        return download_path;
    }

    let mut file_extension = Path::new(file_name)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap()
        // convert extension letters to lower case
        // for example "JPG" will be converted in "jpg"
        .to_lowercase();

    // remove query from file_extension if existed
    // if '?' in file_extension, then file_name contains query components.
    if file_extension.contains('?') {
        file_extension = file_extension.split('?').next().unwrap().to_string();
    }

    // audio formats
    let audio = [
        "act", "aiff", "aac", "amr", "ape", "au", "awb", "dct", "dss", "dvf", "flac", "gsm",
        "iklax", "ivs", "m4a", "m4p", "mmf", "mp3", "mpc", "msv", "ogg", "oga", "opus", "ra",
        "raw", "sln", "tta", "vox", "wav", "wma", "wv",
    ];

    // video formats
    let video = [
        "3g2", "3gp", "asf", "avi", "drc", "flv", "m4v", "mkv", "mng", "mov", "qt", "mp4", "m4p",
        "mpg", "mp2", "mpeg", "mpe", "mpv", "m2v", "mxf", "nsv", "ogv", "rmvb", "roq", "svi",
        "vob", "webm", "wmv", "yuv", "rm",
    ];

    // document formats
    let document = [
        "doc", "docx", "html", "htm", "fb2", "odt", "sxw", "pdf", "ps", "rtf", "tex", "txt",
        "epub", "pub", "mobi", "azw", "azw3", "azw4", "kf8", "chm", "cbt", "cbr", "cbz", "cb7",
        "cba", "ibooks", "djvu", "md",
    ];

    // compressed formats
    let compressed = [
        "a", "ar", "cpio", "shar", "LBR", "iso", "lbr", "mar", "tar", "bz2", "F", "gz", "lz",
        "lzma", "lzo", "rz", "sfark", "sz", "xz", "Z", "z", "infl", "7z", "s7z", "ace", "afa",
        "alz", "apk", "arc", "arj", "b1", "ba", "bh", "cab", "cfs", "cpt", "dar", "dd", "dgc",
        "dmg", "ear", "gca", "ha", "hki", "ice", "jar", "kgb", "lzh", "lha", "lzx", "pac",
        "partimg", "paq6", "paq7", "paq8", "pea", "pim", "pit", "qda", "rar", "rk", "sda", "sea",
        "sen", "sfx", "sit", "sitx", "sqx", "tar.gz", "tgz", "tar.Z", "tar.bz2", "tbz2",
        "tar.lzma", "tlz", "uc", "uc0", "uc2", "ucn", "ur2", "ue2", "uca", "uha", "war", "wim",
        "xar", "xp3", "yz1", "zip", "zipx", "zoo", "zpaq", "zz", "ecc", "par", "par2",
    ];

    if audio.contains(&file_extension.as_str()) {
        download_path.join("Audios")
    }
    // aria2c downloads youtube links file_name with 'videoplayback' name?!
    else if video.contains(&file_extension.as_str()) {
        download_path.join("Videos")
    } else if document.contains(&file_extension.as_str()) {
        download_path.join("Documents")
    } else if compressed.contains(&file_extension.as_str()) {
        download_path.join("Compressed")
    } else {
        download_path.join("Other")
    }
}

// shutdown aria2
#[pyfunction]
pub fn shutDown() -> bool {
    let answer = Runtime::new().unwrap().handle().block_on(async {
        let server_url = SERVER_URL.read().await;
        let client = Client::connect(&server_url, None).await.unwrap();
        client.shutdown().await
    });
    match answer {
        Ok(_) => {
            info!("Aria2 Shutdown: Ok");
            true
        }
        Err(e) => {
            error!("Aria2 Shutdown Error: {e}");
            false
        }
    }
}

// downloadPause pauses download
#[pyfunction]
pub fn downloadPause(gid: &str) -> Option<String> {
    // see aria2 documentation for more information

    // send pause request to aria2.
    let answer = Runtime::new().unwrap().handle().block_on(async {
        let server_url = SERVER_URL.read().await;
        let client = Client::connect(&server_url, None).await.unwrap();
        client.pause(gid).await
    });
    info!("{answer:?} paused");
    match answer {
        Ok(_) => Some("Ok".to_string()),
        Err(_) => None,
    }
}

// downloadUnpause unpauses download
#[pyfunction]
pub fn downloadUnpause(gid: &str) -> Option<String> {
    // send unpause request to aria2
    let answer = Runtime::new().unwrap().handle().block_on(async {
        let server_url = SERVER_URL.read().await;
        let client = Client::connect(&server_url, None).await.unwrap();
        client.unpause(gid).await
    });
    info!("{answer:?} paused");
    match answer {
        Ok(_) => Some("Ok".to_string()),
        Err(_) => None,
    }
}

// limitSpeed limits download speed
#[pyfunction]
pub fn limitSpeed(gid: &str, limit: &str) {
    let mut editedlimit = limit.to_string();
    // convert Mega to Kilo, RPC does not Support floating point numbers.
    if limit != "0" {
        let mut limit_number: f32 = limit[0..limit.len() - 1].parse().unwrap();
        let mut limit_unit = limit.chars().last().unwrap();
        if limit_unit == 'K' {
            limit_number = round(limit_number, 0);
        } else {
            limit_number = round(1024.0 * limit_number, 0);
            limit_unit = 'K';
        }
        editedlimit = format!("{limit_number}{limit_unit}");
    }

    let options = TaskOptions {
        max_download_limit: Some(editedlimit),
        ..Default::default()
    };

    let answer = Runtime::new().unwrap().handle().block_on(async {
        let server_url = SERVER_URL.read().await;
        let client = Client::connect(&server_url, None).await.unwrap();
        client.change_option(gid, options).await
    });

    match answer {
        Ok(_) => info!("Download speed limit value is changed"),
        Err(_) => error!("Speed limitation was unsuccessful"),
    }
}

// this function returns GID of active downloads in list format.
#[pyfunction]
pub fn activeDownloads() -> Vec<String> {
    let answer = Runtime::new().unwrap().handle().block_on(async {
        let server_url = SERVER_URL.read().await;
        let client = Client::connect(&server_url, None).await.unwrap();
        client
            .custom_tell_active(Some(vec!["gid".to_string()]))
            .await
    });

    let answer = match answer {
        Ok(answer) => answer,
        Err(_) => vec![],
    };
    let mut active_gids = vec![];
    for download_dict in answer {
        // add gid to list
        active_gids.push(download_dict.get("gid").unwrap().to_string());
    }
    active_gids
}

// This function returns data and time in string format
// for example >> 2017/09/09 , 13:12:26
#[pyfunction]
pub fn nowDate() -> String {
    let now = Local::now();
    now.format("%Y/%m/%d , %H:%M:%S").to_string()
}

// sigmaTime gets hours and minutes for input.
// and converts hours to minutes and returns summation in minutes
// input format is HH:MM
fn _sigmaTime(time: String) -> i32 {
    let splitedTime: Vec<&str> = time.split(':').collect();
    let hour: i32 = splitedTime[0].parse().unwrap();
    let minute: i32 = splitedTime[1].parse().unwrap();
    hour * 60 + minute
}

// nowTime returns now time in HH:MM format!
fn _nowTime() -> i32 {
    let now_time = Local::now().format("%H:%M");
    _sigmaTime(now_time.to_string())
}
