/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use async_stream::stream;
use pokemon_service_client::model::AttemptCapturingPokemonEvent;
use pokemon_service_client::model::CapturingEvent;
use pokemon_service_client::{Builder, Client, Config};
use rand::Rng;

fn get_pokeball() -> String {
    let random = rand::thread_rng().gen_range(0..100);
    let pokeball = if random < 5 {
        "Master Ball"
    } else if random < 30 {
        "Great Ball"
    } else {
        "Fast Ball"
    };
    pokeball.to_string()
}
fn get_pokemon_to_capture() -> String {
    let pokemons = vec!["Charizard", "Pikachu", "Regieleki"];
    pokemons[rand::thread_rng().gen_range(0..pokemons.len())].to_string()
}

#[tokio::main]
pub async fn main() -> Result<(), ()> {
    let raw_client = Builder::dyn_https()
        .middleware_fn(|mut req| {
            let http_req = req.http_mut();
            let uri = format!("http://localhost:13734{}", http_req.uri().path());
            *http_req.uri_mut() = uri.parse().unwrap();
            req
        })
        .build_dyn();
    let config = Config::builder().build();
    let client = Client::with_config(raw_client, config);

    let mut team = vec![];
    let input_stream = stream! {
        // Always Pikachu
        yield Ok(AttemptCapturingPokemonEvent::Event(
            CapturingEvent::builder().name("Pikachu").pokeball("Master Ball").build()
        ));
        yield Ok(AttemptCapturingPokemonEvent::Event(
            CapturingEvent::builder().name("Regieleki").pokeball("Fast Ball").build()
        ));
        yield Ok(AttemptCapturingPokemonEvent::Event(
            CapturingEvent::builder().name("Charizard").pokeball("Great Ball").build()
        ));
    };

    // Throw many!
    let mut output = client
        .capture_pokemon_operation()
        .events(input_stream.into())
        .send()
        .await
        .unwrap();
    loop {
        match output.events.recv().await {
            Ok(Some(capture)) => {
                let pokemon = capture.as_event().unwrap().name.as_ref().unwrap().clone();
                println!("captured {}", pokemon);
                team.push(pokemon);
            }
            Err(e) => {
                println!("error {:?}", e);
                break;
            }
            Ok(None) => break,
        }
    }

    while team.len() < 6 {
        let pokeball = get_pokeball();
        let pokemon = get_pokemon_to_capture();
        let input_stream = stream! {
            yield Ok(AttemptCapturingPokemonEvent::Event(
                CapturingEvent::builder().name(pokemon).pokeball(pokeball).build()
            ))
        };
        let mut output = client
            .capture_pokemon_operation()
            .events(input_stream.into())
            .send()
            .await
            .unwrap();
        match output.events.recv().await {
            Ok(Some(capture)) => {
                let pokemon = capture.as_event().unwrap().name.as_ref().unwrap().clone();
                println!("captured {}", pokemon);
                team.push(pokemon);
            }
            Err(e) => {
                println!("error {:?}", e);
                break;
            }
            Ok(None) => {}
        }
    }
    println!("Team: {:?}", team);
    Ok(())
}
