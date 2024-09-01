/*
* Copyright (c) 2021, Dr. Spandan Roy
*
* This file is part of mover.
*
* mover is free software: you can redistribute it and/or modify
* it under the terms of the GNU General Public License as published by
* the Free Software Foundation, either version 3 of the License, or
* (at your option) any later version.
*
* mover is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License
* along with mover.  If not, see <https://www.gnu.org/licenses/>.
*/

// API DOCS. Refer to this
// https://yts.mx/api

use reqwest::Error;
use serde::Deserialize;
use urlencoding::encode as encode_url;

#[derive(Deserialize, Debug)]
struct Torrent {
    hash: String,
    quality: String,
    #[serde(rename = "type")]
    type_: String,
    video_codec: String,
    seeds: u32,
    peers: u32,
    size: String,
    size_bytes: u64,
}

// magnet:?xt=urn:btih:TORRENT_HASH&dn=Url+Encoded+Movie+Name&tr=http://track.one:1234/announce&tr=udp://track.two:80

impl Torrent {
    fn get_trackers() -> Vec<String> {
        vec![
            "udp://open.demonii.com:1337/announce",
            "udp://tracker.openbittorrent.com:80",
            "udp://tracker.coppersurfer.tk:6969",
            "udp://glotorrents.pw:6969/announce",
            "udp://tracker.opentrackr.org:1337/announce",
            "udp://torrent.gresille.org:80/announce",
            "udp://p4p.arenabg.com:1337",
            "udp://tracker.leechers-paradise.org:6969",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect()
    }

    pub fn get_magnet_link(&self, name: &str) -> String {
        let dn = format!("{}{}", name, self.quality);
        let trackers = Torrent::get_trackers()
            .iter()
            .map(|x| format!("tr={}", encode_url(x)))
            .collect::<Vec<String>>()
            .join("&");
        let dn = encode_url(&dn);
        let trackers = encode_url(&trackers);
        format!("magnet:?xt=urn:btih:{}&dn={}&{}", self.hash, dn, trackers)
    }
}

#[derive(Deserialize, Debug)]
struct Movie {
    id: u32,
    title: String,
    year: u16,
    summary: String,
    description_full: String,
    background_image: String,
    medium_cover_image: String,
    large_cover_image: String,
    torrents: Vec<Torrent>,
}

#[derive(Deserialize, Debug)]
struct MovieData {
    movies: Option<Vec<Movie>>,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    status: String,
    status_message: String,
    data: Option<MovieData>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Enter a movie name: ");
    let mut query = String::new();
    std::io::stdin().read_line(&mut query).unwrap();
    let query = encode_url(query.trim());
    let url = format!(
        "https://yts.mx/api/v2/list_movies.json?query_term={}&sort_by=download_count",
        query
    );

    let response = reqwest::get(url).await?;
    let api_response: ApiResponse = response.json().await?;

    if api_response.status != "ok" {
        println!("Error: {}", api_response.status_message);
        return Ok(());
    } else {
        match api_response.data {
            Some(data) => {
                if let Some(movies) = data.movies {
                    for movie in movies {
                        println!("Title: {}", movie.title);
                        println!("Year: {}", movie.year);
                        for torrent in movie.torrents {
                            println!("Quality: {}", torrent.quality);
                            println!("Seeds: {}", torrent.seeds);
                            println!("Peers: {}", torrent.peers);
                            println!("Size: {}", torrent.size);
                            println!("Magnet Link: {}", torrent.get_magnet_link(&movie.title));
                            println!();
                        }
                        println!();
                    }
                } else {
                    println!("No movies found.");
                }
            }
            None => println!("No data available."),
        }
    }

    Ok(())
}
