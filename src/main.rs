use std::env;
use reqwest;
use warp::{self, Filter, http::response::Builder};
use once_cell::sync::OnceCell;
use m3u8_rs::playlist::Playlist;

static M3U8_ORIGIN: OnceCell<String> = OnceCell::new();

pub async fn list_todos(path: warp::path::FullPath, query: String) -> Result<impl warp::Reply, std::convert::Infallible> {
    let client = reqwest::Client::new();
    let origin = M3U8_ORIGIN.get().unwrap();
    let url = format!("{}/{}?{}", origin, path.as_str(), query);
    let request = client.get(url);
    let response = request.send().await.unwrap();
    let status = response.status();
    let body = response.bytes().await.unwrap();

    let parsed = m3u8_rs::parse_playlist_res(&body);

    let mut buffer: Vec<u8> = Vec::new();

    match parsed {
        Ok(Playlist::MasterPlaylist(mut pl)) => {
            for variant in pl.variants.iter_mut() {
                variant.uri = format!("{}?{}", variant.uri, query);
            }
            for alternative in pl.alternatives.iter_mut() {
                alternative.uri = alternative.uri.as_ref().map(|uri| format!("{}?{}", uri, query));
            }
            pl.write_to(&mut buffer).unwrap();
        },
        Ok(Playlist::MediaPlaylist(mut pl)) => {
            for segment in pl.segments.iter_mut() {
                segment.uri = format!("{}/{}?{}", origin, segment.uri, query);
            }
            pl.write_to(&mut buffer).unwrap();
        },
        Err(e) => panic!("Error: {:?}", e),
    };

    Ok(Builder::new().status(status).body(buffer))
}

#[tokio::main]
async fn main() {
    M3U8_ORIGIN.set(env::var("M3U8_ORIGIN").expect("M3U8_ORIGIN must be set")).unwrap();

    let index = warp::any()
        .and(warp::path::full())
        .and(warp::query::raw())
        .and_then(list_todos);

    warp::serve(index)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
