#[warn(unused_variables)]
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    routing::get,
    Router,
};
use std::hash::Hash;
use std::hash::Hasher;

use axum::http::{HeaderMap, HeaderValue};
use bytes::Bytes;
use lru::LruCache;
use percent_encoding::percent_decode_str;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use serde::Deserialize;
use std::convert::TryInto;
use std::num::NonZeroUsize;
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

use anyhow::Result;
use std::collections::hash_map::DefaultHasher;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, instrument};

mod engine;
mod pb;
use pb::*;

use engine::{Engine, Photon};
use image::ImageOutputFormat;

#[derive(Deserialize)]
struct Params {
    spec: String,
    url: String,
}

type Cache = Arc<Mutex<LruCache<u64, Bytes>>>;

#[tokio::main]
async fn main() {
    // 初始化tracing
    tracing_subscriber::fmt::init();

    // 引入缓存
    let cache: Cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1024).unwrap())));

    // 构建路由
    let app = Router::new()
        .route("/image/:spec/:url", get(generate))
        .layer(
            ServiceBuilder::new()
                .layer(AddExtensionLayer::new(cache))
                .into_inner(),
        );

    print_test_url("https://images.pexels.com/photos/1562477/pexels-photo-1562477.jpeg?auto=compress&cs=tinysrgb&dpr=3&h=750&w=1260");

    // 运行服务器
    let addr = "127.0.0.1:3000".parse().unwrap();

    // 日志追踪
    tracing::info!("Listening on {:?}", addr);
    // 启动服务器

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn generate(
    Path(Params { spec, url }): Path<Params>,
    Extension(cache): Extension<Cache>,
) -> Result<(HeaderMap, Vec<u8>), StatusCode> {
    let spec: ImageSpec = spec
        .as_str()
        .try_into()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let url: &str = &percent_decode_str(&url).decode_utf8_lossy();

    let data = retrieve_image(&url, cache)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut engine: Photon = data
        .try_into()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    engine.apply(&spec.specs);

    let image = engine.generate(ImageOutputFormat::Jpeg(85));

    info!("Finished processing: image size {}", image.len());
    let mut headers = HeaderMap::new();
    headers.insert("content-type", HeaderValue::from_static("image/jpeg"));

    Ok((headers, image))
}

#[instrument(level = "info", skip(cache))]
async fn retrieve_image(url: &str, cache: Cache) -> Result<Bytes> {
    let mut hasher = DefaultHasher::new();

    url.hash(&mut hasher);
    let key = hasher.finish();

    let g = &mut cache.lock().await;

    let data = match g.get(&key) {
        Some(v) => {
            info!("Match cache {}", key);
            v.to_owned()
        }
        None => {
            info!("Retrieve url");
            let resp = reqwest::get(url).await?;

            let data = resp.bytes().await?;
            g.put(key, data.clone());

            data
        }
    };

    Ok(data)
}

// 测试案例

fn print_test_url(url: &str) {
    use std::borrow::Borrow;
    let spec1 = Spec::new_resize(500, 800, resize::SampleFilter::CatmullRom);
    let spec2 = Spec::new_watermark(20, 20);
    let spec3 = Spec::new_filter(filter::Filter::Marine);
    let image_spec = ImageSpec::new(vec![spec1, spec2, spec3]);
    let s: String = image_spec.borrow().into();
    let test_image = percent_encode(url.as_bytes(), NON_ALPHANUMERIC).to_string();
    println!("test url: http://localhost:3000/image/{}/{}", s, test_image);
}
