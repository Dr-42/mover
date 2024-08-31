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

use reqwest::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Movie {
    title: String,
    year: u16,
    rating: f32,
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
    let query = query.replace(" ", "+");
    let url = format!(
        "https://yts.mx/api/v2/list_movies.json?query_term={}&sort_by=download_count",
        query
    );

    let response = reqwest::get(url).await?;
    let api_response: ApiResponse = response.json().await?;

    match api_response.data {
        Some(data) => {
            if let Some(movies) = data.movies {
                for movie in movies {
                    println!("Title: {}", movie.title);
                    println!("Year: {}", movie.year);
                    println!("Rating: {}", movie.rating);
                    println!("");
                }
            } else {
                println!("No movies found.");
            }
        }
        None => println!("No data available."),
    }

    Ok(())
}
