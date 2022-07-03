use crate::cache::map::{CACHEDATA, CACHEJSON};
use actix_web::http::header::{ACCEPT_LANGUAGE, CONTENT_TYPE, USER_AGENT};
use actix_web::http::StatusCode;
use actix_web::web::{self, Bytes};
use awc::Client;
use core::time::Duration;
use serde_json::value::Value;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::sync::Arc;

pub async fn getplayer(
    client: &web::Data<Client>,
    vid: &String,
) -> Result<Arc<HashMap<String, Value>>, Box<dyn Error>> {
    let limit = 5 << 20;
    let real = || async {
        if let Ok(res) = getnetplayer(client, vid, limit).await {
            return Some(res);
        }
        return None;
    };
    let item = CACHEJSON.load_or_store(vid, real, 3600).await;
    if let Some(res) = item {
        return Ok(res);
    }
    getnetplayer(client, vid, limit).await
}

async fn getnetplayer(
    client: &web::Data<Client>,
    vid: &String,
    limit: usize,
) -> Result<Arc<HashMap<String, Value>>, Box<dyn Error>> {
    let video_url = "https://youtubei.googleapis.com/youtubei/v1/player?key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
    let req = serde_json::json!({
        "videoId": vid,
        "context": {
            "client": {
                "clientName": "Android",
                "clientVersion": "16.13.35"
            }
        }
    });

    let mut response = client
        .post(video_url)
        .timeout(Duration::from_secs(10))
        .insert_header((USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.0.4606.81 Safari/537.36"))
        .insert_header((CONTENT_TYPE, "application/json"))
        .insert_header((ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9,en;q=0.8"))
        .send_json(&req)
        .await?;
    match response.status() {
        StatusCode::OK => {
            let res = response
                .json::<HashMap<String, Value>>()
                .limit(limit)
                .await?;
            Ok(Arc::new(res))
        }
        _ => {
            println!("status: failed {} {}", vid, response.status());
            let res = response.body().limit(limit).await?;
            println!("{:?}", res);
            Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("{} {}", vid, response.status()),
            )))
        }
    }
}

pub async fn getdata(
    client: &web::Data<Client>,
    url: &String,
    ttl: u64,
    limit: u32,
) -> Result<Arc<Bytes>, Box<dyn Error>> {
    let real = || async {
        if let Ok(res) = req_get(client, url, limit).await {
            return Some(res);
        }
        return None;
    };
    let item = CACHEDATA.load_or_store(url, real, ttl).await;
    if let Some(res) = item {
        return Ok(res);
    }
    req_get(client, url, limit).await
}

pub async fn req_get(
    client: &web::Data<Client>,
    url: &String,
    limit: u32,
) -> Result<Arc<Bytes>, Box<dyn Error>> {
    let mut response = client
        .get(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await?;
    let res = response.body().limit(limit as usize).await?;
    match response.status() {
        StatusCode::OK => Ok(Arc::new(res)),
        _ => {
            println!("status: failed {} {}", url, response.status());
            println!("{:?}", res);
            Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("{} {}", url, response.status()),
            )))
        }
    }
}
