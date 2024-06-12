use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, TimestampSeconds};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub enabled_features: Vec<String>,

    pub version: String,
}

/// Full status of a task.
///
/// <https://aria2.github.io/manual/en/html/aria2c.html#aria2.tellStatus>
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// GID of the download.
    pub gid: String,

    pub status: TaskStatus,

    #[serde_as(as = "DisplayFromStr")]
    pub total_length: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub completed_length: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub upload_length: u64,

    /// Hexadecimal representation of the download progress.
    ///
    /// The highest bit corresponds to the piece at index 0.
    ///
    /// Any set bits indicate loaded pieces, while
    /// unset bits indicate not yet loaded and/or missing pieces.
    ///
    /// Any overflow bits at the end are set to zero.
    ///
    /// When the download was not started yet,
    /// this key will not be included in the response.
    pub bitfield: Option<String>,

    #[serde_as(as = "DisplayFromStr")]
    pub download_speed: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub upload_speed: u64,

    /// InfoHash. BitTorrent only
    pub info_hash: Option<String>,

    #[serde_as(as = "Option<DisplayFromStr>")]
    pub num_seeders: Option<u64>,

    /// true if the local endpoint is a seeder. Otherwise false. BitTorrent only.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub seeder: Option<bool>,

    #[serde_as(as = "DisplayFromStr")]
    pub piece_length: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub num_pieces: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub connections: u64,

    pub error_code: Option<String>,

    pub error_message: Option<String>,
    /// List of GIDs which are generated as the result of this download.
    ///
    /// For example, when aria2 downloads a Metalink file,
    /// it generates downloads described in the Metalink (see the --follow-metalink option).
    ///
    /// This value is useful to track auto-generated downloads.
    ///
    /// If there are no such downloads, this key will not be included in the response.
    pub followed_by: Option<Vec<String>>,

    /// The reverse link for followedBy.
    ///
    /// A download included in followedBy has this object's GID in its following value.
    pub following: Option<String>,

    /// GID of a parent download.
    ///
    /// Some downloads are a part of another download.
    ///
    /// For example, if a file in a Metalink has BitTorrent resources,
    /// the downloads of ".torrent" files are parts of that parent.
    ///
    /// If this download has no parent, this key will not be included in the response.
    pub belongs_to: Option<String>,

    pub dir: String,

    pub files: Vec<File>,

    pub bittorrent: Option<BittorrentStatus>,

    /// The number of verified number of bytes while the files are being hash checked.
    ///
    /// This key exists only when this download is being hash checked.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub verified_length: Option<u64>,

    /// `true` if this download is waiting for the hash check in a queue.
    ///
    /// This key exists only when this download is in the queue.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub verify_integrity_pending: Option<bool>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BittorrentStatus {
    pub announce_list: Vec<Vec<String>>,

    pub comment: Option<String>,

    #[serde_as(as = "Option<TimestampSeconds<i64>>")]
    pub creation_date: Option<DateTime<Utc>>,

    pub mode: Option<BitTorrentFileMode>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BitTorrentFileMode {
    Single,
    Multi,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct File {
    #[serde_as(as = "DisplayFromStr")]
    pub index: u64,

    pub path: String,

    #[serde_as(as = "DisplayFromStr")]
    pub length: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub completed_length: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub selected: bool,

    pub uris: Vec<Uri>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Uri {
    pub status: UriStatus,

    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UriStatus {
    Used,
    Waiting,
}

/// Task status returned by `aria2.tellStatus`.
///
/// `Active` for currently downloading/seeding downloads.
///
/// `Waiting` for downloads in the queue; download is not started.
///
/// `Paused` for paused downloads.
///
/// `Error` for downloads that were stopped because of error.
///
/// `Complete` for stopped and completed downloads.
///
/// `Removed` for the downloads removed by user.
///
/// <https://aria2.github.io/manual/en/html/aria2c.html#aria2.tellStatus>
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Active,
    Waiting,
    Paused,
    Error,
    Complete,
    Removed,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Peer {
    #[serde_as(as = "DisplayFromStr")]
    pub am_choking: bool,

    pub bitfield: String,

    #[serde_as(as = "DisplayFromStr")]
    pub download_speed: u64,

    pub ip: String,

    #[serde_as(as = "DisplayFromStr")]
    pub peer_choking: bool,

    pub peer_id: String,

    #[serde_as(as = "DisplayFromStr")]
    pub port: u16,

    #[serde_as(as = "DisplayFromStr")]
    pub seeder: bool,

    #[serde_as(as = "DisplayFromStr")]
    pub upload_speed: u64,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalStat {
    #[serde_as(as = "DisplayFromStr")]
    pub download_speed: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub upload_speed: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub num_active: i32,

    #[serde_as(as = "DisplayFromStr")]
    pub num_waiting: i32,

    #[serde_as(as = "DisplayFromStr")]
    pub num_stopped: i32,

    #[serde_as(as = "DisplayFromStr")]
    pub num_stopped_total: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub session_id: String,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GetServersResult {
    #[serde_as(as = "DisplayFromStr")]
    pub index: i32,

    pub servers: Vec<Server>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub uri: String,

    pub current_uri: String,

    #[serde_as(as = "DisplayFromStr")]
    pub download_speed: u64,
}
