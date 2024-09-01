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

use librqbit::{AddTorrent, Session};
use serde::Deserialize;
use std::time::Duration;
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

    pub async fn download(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let session = Session::new("/home/spandan/Downloads".into())
            .await
            .unwrap();
        let handle = session
            .add_torrent(
                AddTorrent::from_url(self.get_magnet_link(name)),
                None, // options
            )
            .await
            .unwrap()
            .into_handle()
            .unwrap();
        tokio::spawn({
            let h = handle.clone();
            async move {
                loop {
                    println!("{}", h.stats());
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        });
        handle.wait_until_completed().await?;
        Ok(())
    }
}
