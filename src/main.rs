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

use serde::Deserialize;
use std::error::Error;
use std::io::Write;
use urlencoding::encode as encode_url;

pub mod key_controls;
pub mod media_player;
pub mod torr;

use torr::Torrent;

#[allow(dead_code)]
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
async fn main() -> Result<(), Box<dyn Error>> {
    print!("Enter a movie name: ");
    std::io::stdout().flush().unwrap();
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
                    for (i, movie) in movies.iter().enumerate() {
                        println!("{}. {} ({})", i + 1, movie.title, movie.year);
                    }
                    print!("Enter the movie number to get magnets: ");
                    std::io::stdout().flush().unwrap();
                    let mut choice = String::new();
                    std::io::stdin().read_line(&mut choice).unwrap();
                    let choice: usize = choice.trim().parse().unwrap();
                    if choice > movies.len() {
                        println!("Invalid choice.");
                        return Ok(());
                    }
                    let movie = &movies[choice - 1];
                    println!("Title: {}", movie.title);
                    println!("Year: {}", movie.year);
                    for (i, torrent) in movie.torrents.iter().enumerate() {
                        println!(
                            "{}. Quality: {} Seeds: {} Peers: {} Size: {} Codec: {} Type: {}",
                            i + 1,
                            torrent.quality,
                            torrent.seeds,
                            torrent.peers,
                            torrent.size,
                            torrent.video_codec,
                            torrent.type_,
                        );
                    }
                    print!("Enter the torrent number to download: ");
                    std::io::stdout().flush().unwrap();
                    let mut choice = String::new();
                    std::io::stdin().read_line(&mut choice).unwrap();
                    let choice: usize = choice.trim().parse().unwrap();
                    if choice > movie.torrents.len() {
                        println!("Invalid choice.");
                        return Ok(());
                    }
                    let torrent = &movie.torrents[choice - 1];
                    let movie_path = torrent.download(&movie.title).await?;
                    println!();
                    println!("Download complete.");

                    print!("Do you want to play the movie? (y/n): ");
                    std::io::stdout().flush().unwrap();
                    let mut choice = String::new();
                    std::io::stdin().read_line(&mut choice).unwrap();
                    if choice.trim() == "y" {
                        let (finished, time) = media_player::run(movie_path, 0.0);
                        if finished {
                            println!("Movie finished.");
                        } else {
                            println!("Movie stopped at {}", time);
                        }
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
