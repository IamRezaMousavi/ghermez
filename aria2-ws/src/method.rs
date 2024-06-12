use crate::{
    error,
    options::TaskOptions,
    response,
    utils::{value_into_vec, PushExt},
    Callbacks, Client, InnerClient, Result,
};
use base64::prelude::*;
use serde::Serialize;
use serde_json::{json, to_value, Map, Value};
use snafu::prelude::*;

/// The parameter `how` in `changePosition`.
///
/// <https://aria2.github.io/manual/en/html/aria2c.html#aria2.changePosition>
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub enum PositionHow {
    #[serde(rename = "POS_SET")]
    Set,
    #[serde(rename = "POS_CUR")]
    Cur,
    #[serde(rename = "POS_END")]
    End,
}

impl InnerClient {
    async fn custom_tell_multi(
        &self,
        method: &str,
        offset: i32,
        num: i32,
        keys: Option<Vec<String>>,
    ) -> Result<Vec<Map<String, Value>>> {
        let mut params = value_into_vec(json!([offset, num]));
        params.push_some(keys)?;
        self.call_and_wait(method, params).await
    }

    pub async fn get_version(&self) -> Result<response::Version> {
        self.call_and_wait("getVersion", vec![]).await
    }

    async fn returning_gid(&self, method: &str, gid: &str) -> Result<()> {
        self.call_and_wait::<String>(method, vec![Value::String(gid.to_string())])
            .await?;
        Ok(())
    }

    pub async fn remove(&self, gid: &str) -> Result<()> {
        self.returning_gid("remove", gid).await
    }

    pub async fn force_remove(&self, gid: &str) -> Result<()> {
        self.returning_gid("forceRemove", gid).await
    }

    pub async fn pause(&self, gid: &str) -> Result<()> {
        self.returning_gid("pause", gid).await
    }

    pub async fn pause_all(&self) -> Result<()> {
        self.call_and_wait::<String>("pauseAll", vec![]).await?;
        Ok(())
    }

    pub async fn force_pause(&self, gid: &str) -> Result<()> {
        self.returning_gid("forcePause", gid).await
    }

    pub async fn force_pause_all(&self) -> Result<()> {
        self.call_and_wait::<String>("forcePauseAll", vec![])
            .await?;
        Ok(())
    }

    pub async fn unpause(&self, gid: &str) -> Result<()> {
        self.returning_gid("unpause", gid).await
    }

    pub async fn unpause_all(&self) -> Result<()> {
        self.call_and_wait::<String>("unpauseAll", vec![]).await?;
        Ok(())
    }

    pub async fn custom_tell_status(
        &self,
        gid: &str,
        keys: Option<Vec<String>>,
    ) -> Result<Map<String, Value>> {
        let mut params = vec![Value::String(gid.to_string())];
        params.push_some(keys)?;
        self.call_and_wait("tellStatus", params).await
    }

    pub async fn tell_status(&self, gid: &str) -> Result<response::Status> {
        self.call_and_wait("tellStatus", vec![Value::String(gid.to_string())])
            .await
    }

    pub async fn get_uris(&self, gid: &str) -> Result<Vec<response::Uri>> {
        self.call_and_wait("getUris", vec![Value::String(gid.to_string())])
            .await
    }

    pub async fn get_files(&self, gid: &str) -> Result<Vec<response::File>> {
        self.call_and_wait("getFiles", vec![Value::String(gid.to_string())])
            .await
    }

    pub async fn get_peers(&self, gid: &str) -> Result<Vec<response::Peer>> {
        self.call_and_wait("getPeers", vec![Value::String(gid.to_string())])
            .await
    }

    pub async fn get_servers(&self, gid: &str) -> Result<Vec<response::GetServersResult>> {
        self.call_and_wait("getServers", vec![Value::String(gid.to_string())])
            .await
    }

    pub async fn tell_active(&self) -> Result<Vec<response::Status>> {
        self.call_and_wait("tellActive", vec![]).await
    }

    pub async fn tell_waiting(&self, offset: i32, num: i32) -> Result<Vec<response::Status>> {
        self.call_and_wait("tellWaiting", value_into_vec(json!([offset, num])))
            .await
    }

    pub async fn tell_stopped(&self, offset: i32, num: i32) -> Result<Vec<response::Status>> {
        self.call_and_wait("tellStopped", value_into_vec(json!([offset, num])))
            .await
    }

    pub async fn custom_tell_active(
        &self,
        keys: Option<Vec<String>>,
    ) -> Result<Vec<Map<String, Value>>> {
        let mut params = Vec::new();
        params.push_some(keys)?;
        self.call_and_wait("tellActive", params).await
    }

    pub async fn custom_tell_waiting(
        &self,
        offset: i32,
        num: i32,
        keys: Option<Vec<String>>,
    ) -> Result<Vec<Map<String, Value>>> {
        self.custom_tell_multi("tellWaiting", offset, num, keys)
            .await
    }

    pub async fn custom_tell_stopped(
        &self,
        offset: i32,
        num: i32,
        keys: Option<Vec<String>>,
    ) -> Result<Vec<Map<String, Value>>> {
        self.custom_tell_multi("tellStopped", offset, num, keys)
            .await
    }

    pub async fn change_position(&self, gid: &str, pos: i32, how: PositionHow) -> Result<i32> {
        let params = value_into_vec(json!([gid, pos, how]));
        self.call_and_wait("changePosition", params).await
    }

    /// # Returns
    /// This method returns a list which contains two integers.
    ///
    /// The first integer is the number of URIs deleted.
    /// The second integer is the number of URIs added.
    pub async fn change_uri(
        &self,
        gid: &str,
        file_index: i32,
        del_uris: Vec<String>,
        add_uris: Vec<String>,
        position: Option<i32>,
    ) -> Result<(i32, i32)> {
        let mut params = value_into_vec(json!([gid, file_index, del_uris, add_uris]));
        params.push_some(position)?;
        self.call_and_wait("changeUri", params).await
    }

    pub async fn get_option(&self, gid: &str) -> Result<TaskOptions> {
        self.call_and_wait("getOption", vec![Value::String(gid.to_string())])
            .await
    }

    pub async fn change_option(&self, gid: &str, options: TaskOptions) -> Result<()> {
        self.call_and_wait(
            "changeOption",
            vec![
                Value::String(gid.to_string()),
                to_value(options).context(error::JsonSnafu)?,
            ],
        )
        .await
    }

    pub async fn get_global_option(&self) -> Result<TaskOptions> {
        self.call_and_wait("getGlobalOption", vec![]).await
    }

    pub async fn change_global_option(&self, options: TaskOptions) -> Result<()> {
        self.call_and_wait(
            "changeGlobalOption",
            vec![to_value(options).context(error::JsonSnafu)?],
        )
        .await
    }

    pub async fn get_global_stat(&self) -> Result<response::GlobalStat> {
        self.call_and_wait("getGlobalStat", vec![]).await
    }

    pub async fn purge_download_result(&self) -> Result<()> {
        self.call_and_wait::<String>("purgeDownloadResult", vec![])
            .await?;
        Ok(())
    }

    pub async fn remove_download_result(&self, gid: &str) -> Result<()> {
        self.call_and_wait::<String>("removeDownloadResult", vec![Value::String(gid.to_string())])
            .await?;
        Ok(())
    }

    pub async fn get_session_info(&self) -> Result<response::SessionInfo> {
        self.call_and_wait("getSessionInfo", vec![]).await
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.call_and_wait::<String>("shutdown", vec![]).await?;
        Ok(())
    }

    pub async fn force_shutdown(&self) -> Result<()> {
        self.call_and_wait::<String>("forceShutdown", vec![])
            .await?;
        Ok(())
    }

    pub async fn save_session(&self) -> Result<()> {
        self.call_and_wait::<String>("saveSession", vec![]).await?;
        Ok(())
    }
}

impl Client {
    async fn add_callbacks_option(&self, gid: &str, callbacks: Option<Callbacks>) {
        if let Some(callbacks) = callbacks {
            self.add_callbacks(gid.to_string(), callbacks);
        }
    }

    pub async fn add_uri(
        &self,
        uris: Vec<String>,
        options: Option<TaskOptions>,
        position: Option<u32>,
        callbacks: Option<Callbacks>,
    ) -> Result<String> {
        let mut params = vec![to_value(uris).context(error::JsonSnafu)?];
        params.push_else(options, json!({}))?;
        params.push_some(position)?;

        let gid: String = self.call_and_wait("addUri", params).await?;
        self.add_callbacks_option(&gid, callbacks).await;
        Ok(gid)
    }

    pub async fn add_torrent(
        &self,
        torrent: impl AsRef<[u8]>,
        uris: Option<Vec<String>>,
        options: Option<TaskOptions>,
        position: Option<u32>,
        callbacks: Option<Callbacks>,
    ) -> Result<String> {
        let mut params = vec![Value::String(BASE64_STANDARD.encode(torrent))];
        params.push_else(uris, json!([]))?;
        params.push_else(options, json!({}))?;
        params.push_some(position)?;

        let gid: String = self.call_and_wait("addTorrent", params).await?;
        self.add_callbacks_option(&gid, callbacks).await;
        Ok(gid)
    }

    pub async fn add_metalink(
        &self,
        metalink: impl AsRef<[u8]>,
        options: Option<TaskOptions>,
        position: Option<u32>,
        callbacks: Option<Callbacks>,
    ) -> Result<String> {
        let mut params = vec![Value::String(BASE64_STANDARD.encode(metalink))];
        params.push_else(options, json!({}))?;
        params.push_some(position)?;

        let gid: String = self.call_and_wait("addMetalink", params).await?;
        self.add_callbacks_option(&gid, callbacks).await;
        Ok(gid)
    }
}
