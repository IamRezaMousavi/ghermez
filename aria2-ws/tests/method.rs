use std::time::Duration;

use aria2_ws::{Client, TaskOptions};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::{serde_as, skip_serializing_none};
use test_log::test;

async fn test_global(c: &Client) {
    let r = c.get_version().await.unwrap();
    println!("{:?}\n", r);

    let r = c.get_global_option().await.unwrap();
    println!("{:?}\n", r);

    let r = c.get_global_stat().await.unwrap();
    println!("{:?}\n", r);

    let r = c.get_global_option().await.unwrap();
    println!("{:?}\n", r);

    let r = c.get_session_info().await.unwrap();
    println!("{:?}\n", r);

    let r = c.tell_active().await.unwrap();
    println!("{:?}\n", r);

    let r = c.tell_stopped(0, 100).await.unwrap();
    println!("{:?}\n", r);

    let r = c.tell_waiting(0, 100).await.unwrap();
    println!("{:?}\n", r);
}

async fn test_metadata(c: &Client, gid: &str) {
    let r = c.get_option(gid).await.unwrap();
    println!("{:?}\n", r);

    let r = c.get_files(gid).await.unwrap();
    println!("{:?}\n", r);

    // let r = c.get_peers(gid).await.unwrap();
    // println!("{:?}\n", r);

    let r = c.get_servers(gid).await.unwrap();
    println!("{:?}\n", r);

    let r = c.get_uris(gid).await.unwrap();
    println!("{:?}\n", r);
}

async fn sleep(secs: u64) {
    tokio::time::sleep(Duration::from_secs(secs)).await;
}

#[test(tokio::test)]
async fn global() {
    sleep(3).await;
    let c = Client::connect("ws://localhost:6800/jsonrpc", None)
        .await
        .unwrap();
    test_global(&c).await;
}

#[test(tokio::test)]
async fn torrent() {
    let c = Client::connect("ws://localhost:6800/jsonrpc", None)
        .await
        .unwrap();

    let options = TaskOptions {
        max_download_limit: Some("100K".to_string()),
        extra_options: json!({
            "file-allocation": "none",
        })
        .as_object()
        .unwrap()
        .clone(),
        ..Default::default()
    };

    let gid = c
        .add_uri(vec!["magnet:?xt=urn:btih:9b4c1489bfccd8205d152345f7a8aad52d9a1f57&dn=archlinux-2022.05.01-x86_64.iso".to_string()], Some(options), None, None)
        .await.unwrap();

    sleep(10).await;

    // test_global(&c).await.unwrap();
    test_metadata(&c, &gid).await;
    c.remove(&gid).await.unwrap();
    c.remove_download_result(&gid).await.unwrap();
}

#[test(tokio::test)]
async fn http() {
    let c = Client::connect("ws://localhost:6800/jsonrpc", None)
        .await
        .unwrap();

    let options = TaskOptions {
        max_download_limit: Some("100K".to_string()),
        extra_options: json!({
            "file-allocation": "none",
        })
        .as_object()
        .unwrap()
        .clone(),
        ..Default::default()
    };

    let gid = c
        .add_uri(
            vec!["https://mirror.hoster.kz/archlinux/iso/latest/archlinux-x86_64.iso".to_string()],
            Some(options),
            None,
            None,
        )
        .await
        .unwrap();

    sleep(10).await;

    // test_global(&c).await.unwrap();
    test_metadata(&c, &gid).await;
    c.remove(&gid).await.unwrap();
    c.remove_download_result(&gid).await.unwrap();
}

use serde_with::DisplayFromStr;
#[serde_as]
#[skip_serializing_none]
#[derive(Deserialize, Serialize, Debug)]
struct A {
    #[serde_as(as = "Option<DisplayFromStr>")]
    a: Option<u32>,
    b: (i32, i32),
}

#[test]
fn serde_test() {
    let a = A { a: None, b: (1, 2) };
    let j = serde_json::to_string(&a).unwrap();
    println!("{}", j);
    let a = serde_json::from_str::<A>(&j).unwrap();
    println!("{:?}", a);
}
