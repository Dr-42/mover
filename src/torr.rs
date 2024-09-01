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

use librqbit::{AddTorrent, AddTorrentOptions, Session, TorrentMetaV1File};
use serde::Deserialize;
use std::{io::Write, path::PathBuf, time::Duration};
use urlencoding::encode as encode_url;

#[derive(Deserialize, Debug)]
pub struct Torrent {
    pub hash: String,
    pub quality: String,
    pub url: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub video_codec: String,
    pub seeds: u32,
    pub peers: u32,
    pub size: String,
    pub size_bytes: u64,
}

// magnet:?xt=urn:btih:TORRENT_HASH&dn=Url+Encoded+Movie+Name&tr=http://track.one:1234/announce&tr=udp://track.two:80

impl Torrent {
    fn get_trackers() -> Vec<String> {
        [
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

    pub async fn download(&self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        const DOWNLOAD_DIR: &str = "/home/spandan/Downloads";
        let session = Session::new(DOWNLOAD_DIR.into()).await.unwrap();
        let handle = session
            .add_torrent(
                AddTorrent::from_url(self.get_magnet_link(name)),
                Some(AddTorrentOptions {
                    overwrite: true,
                    ..Default::default()
                }), // options
            )
            .await
            .unwrap()
            .into_handle()
            .unwrap();
        tokio::spawn({
            let h = handle.clone();
            async move {
                loop {
                    print!("{}", h.stats());
                    std::io::stdout().flush().unwrap();
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    // Reset the cursor to the beginning of the line
                    print!("\r");
                    std::io::stdout().flush().unwrap();
                }
            }
        });
        handle.wait_until_completed().await?;
        //Ok(())
        let info = handle.shared().info.clone();
        let folder = info.name.unwrap();

        let video_exts = ["mp4", "mkv", "avi", "flv", "wmv", "mov"];
        let subtitle_exts = ["srt"];

        // Delete all files except video and subtitle files
        for entry in std::fs::read_dir(format!("{}/{}", DOWNLOAD_DIR, folder))? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                std::fs::remove_dir_all(path)?;
            } else {
                let ext = path.extension();
                if ext.is_none() {
                    std::fs::remove_file(path)?;
                    continue;
                }
                let ext = ext.unwrap().to_str().unwrap();
                if !video_exts.contains(&ext) && !subtitle_exts.contains(&ext) {
                    std::fs::remove_file(path)?;
                }
            }
        }

        // Get all files in the folder
        let mut files = Vec::new();
        for entry in std::fs::read_dir(format!("{}/{}", DOWNLOAD_DIR, folder))? {
            let entry = entry?;
            let path = entry.path();
            files.push(path);
        }
        // If there is only one video file, return it
        for file in &files {
            let ext = file.extension().unwrap().to_str().unwrap();
            if video_exts.contains(&ext) {
                return Ok(file.to_str().unwrap().to_string());
            }
        }
        // If there are multiple video files, return the largest one
        let mut largest = 0;
        let mut largest_file = PathBuf::new();
        for file in &files {
            let metadata = std::fs::metadata(file)?;
            if metadata.len() > largest {
                largest = metadata.len();
                largest_file = file.clone();
            }
        }
        Ok(largest_file.to_str().unwrap().to_string())
    }
}
