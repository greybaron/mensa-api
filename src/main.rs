use constants::{CANTEEN_MAP, CANTEEN_MAP_INV};
use openmensa_funcs::init_openmensa_canteenlist;
use std::env;
use tokio::{net::TcpListener, sync::broadcast};

mod constants;
mod cronjobs;
mod db_operations;
mod openmensa_funcs;
mod routes;
mod services;
mod stuwe_request_funcs;
mod types;
use cronjobs::{start_canteen_cache_job, update_cache};
use db_operations::{check_or_create_db_tables, get_canteens_from_db};
use stuwe_request_funcs::invert_map;

#[tokio::main]
async fn main() {
    if env::var(pretty_env_logger::env_logger::DEFAULT_FILTER_ENV).is_err() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init_timed();
    log::info!("Starting Mensa API...");

    //// DB setup
    check_or_create_db_tables().unwrap();

    {
        let canteens = get_canteens_from_db().await.unwrap();
        *CANTEEN_MAP_INV.write().unwrap() = invert_map(&canteens);
        *CANTEEN_MAP.write().unwrap() = canteens;
    }

    // stuwe_request_funcs::_run_benchmark().await.unwrap();
    // return;
    // let _old = "{\"canteen_id\":106,\"meal_groups\":[{\"meal_type\":\"Veganes Gericht\",\"sub_meals\":[{\"name\":\"Nasi Goreng mit Seitan\",\"additional_ingredients\":[],\"allergens\":\"Konservierungsstoff, glutenhaltiges Getreide, Soja, Sellerie, Sesam, Weizen\",\"variations\":null,\"price\":\"3,50 € / 5,95 € / 7,90 €\"}]},{\"meal_type\":\"Vegetarisches Gericht\",\"sub_meals\":[{\"name\":\"Knuspriges Milchschnitzel \\\"Toscana\\\" (Tomate-Käse-Füllung)\",\"additional_ingredients\":[],\"allergens\":\"glutenhaltiges Getreide, Eier, Milch/ Milchzucker, Weizen, Gerste, Hafer\",\"variations\":null,\"price\":\"2,65 € / 5,10 € / 7,00 €\"}]},{\"meal_type\":\"Fleischgericht\",\"sub_meals\":[{\"name\":\"Putengyros\",\"additional_ingredients\":[\"Zaziki\",\"Balkangemüse\",\"Pommes frites\"],\"allergens\":\"Milch/ Milchzucker\",\"variations\":null,\"price\":\"3,70 € / 5,95 € / 7,85 €\"}]},{\"meal_type\":\"Pastateller\",\"sub_meals\":[{\"name\":\"Pastateller\",\"additional_ingredients\":[],\"allergens\":null,\"variations\":[{\"name\":\"Tomatensoße mit Hähnchenfleisch und Paprika\",\"allergens_and_add\":\"glutenhaltiges Getreide, Weizen\"},{\"name\":\"Tomatensoße mit Kräutern\",\"allergens_and_add\":\"glutenhaltiges Getreide, Weizen\"},{\"name\":\"Bolognese aus Tofu\",\"allergens_and_add\":\"glutenhaltiges Getreide, Soja, Sellerie, Weizen\"},{\"name\":\"Tomatensoße mit Schinken\",\"allergens_and_add\":\"Konservierungsstoff, Antioxidationsmittel, Phosphat, glutenhaltiges Getreide, Weizen\"},{\"name\":\"Käse-Tomatensoße \\\"Parmarosa\\\"\",\"allergens_and_add\":\"glutenhaltiges Getreide, Eier, Milch/ Milchzucker, Weizen\"}],\"price\":\"2,10 € / 4,60 € / 6,35 €\"}]},{\"meal_type\":\"WOK\",\"sub_meals\":[{\"name\":\"Asiatische Gemüsepfanne mit Mie Nudeln\",\"additional_ingredients\":[\"Like Chicken Nuggets\",\"Schweinefleisch in Kokosmilch\"],\"allergens\":\"Konservierungsstoff, glutenhaltiges Getreide, Soja, Weizen\",\"variations\":null,\"price\":\"3,50 € / 5,80 € / 7,40 €\"},{\"name\":\"Asiatische Gemüsepfanne mit Mie Nudeln\",\"additional_ingredients\":[],\"allergens\":\"Konservierungsstoff, glutenhaltiges Getreide, Soja, Weizen\",\"variations\":null,\"price\":\"2,50 € / 4,80 € / 6,60 €\"}]},{\"meal_type\":\"Gemüsebeilage\",\"sub_meals\":[{\"name\":\"Balkangemüse\",\"additional_ingredients\":[],\"allergens\":null,\"variations\":null,\"price\":\"0,55 € / 1,10 € / 1,50 €\"},{\"name\":\"Bohnengemüse\",\"additional_ingredients\":[],\"allergens\":null,\"variations\":null,\"price\":\"0,55 € / 1,10 € / 1,50 €\"}]},{\"meal_type\":\"Sättigungsbeilage\",\"sub_meals\":[{\"name\":\"Pommes frites\",\"additional_ingredients\":[],\"allergens\":null,\"variations\":null,\"price\":\"0,75 € / 1,35 € / 1,75 €\"},{\"name\":\"Dillkartoffeln\",\"additional_ingredients\":[],\"allergens\":null,\"variations\":null,\"price\":\"0,55 € / 1,10 € / 1,50 €\"}]}]}";
    // let old = serde_json::from_str::<types::CanteenMealsDay>(_old).unwrap();
    // let _new = "{\"canteen_id\":106,\"meal_groups\":[{\"meal_type\":\"Veganes Gericht\",\"sub_meals\":[{\"name\":\"Nasi Goreng mit Seitan\",\"additional_ingredients\":[],\"allergens\":\"Konservierungsstoff, glutenhaltiges Getreide, Soja, Sellerie, Sesam, Weizen\",\"variations\":null,\"price\":\"3,50 € / 5,95 € / 7,90 €\"}]},{\"meal_type\":\"Vegetarisches Gericht\",\"sub_meals\":[{\"name\":\"Knuspriges Milchschnitzel \\\"Toscana\\\" (Tomate-Käse-Füllung)\",\"additional_ingredients\":[],\"allergens\":\"glutenhaltiges Getreide, Eier, Milch/ Milchzucker, Weizen, Gerste, Hafer\",\"variations\":null,\"price\":\"2,65 € / 5,10 € / 7,00 €\"}]},{\"meal_type\":\"Fleischgericht\",\"sub_meals\":[{\"name\":\"Putengyros\",\"additional_ingredients\":[\"Zaziki\",\"Balkangemüse\",\"Pommes frites\"],\"allergens\":\"Milch/ Milchzucker\",\"variations\":null,\"price\":\"3,70 € / 5,95 € / 7,85 €\"}]},{\"meal_type\":\"Pastateller\",\"sub_meals\":[{\"name\":\"Pastateller\",\"additional_ingredients\":[],\"allergens\":null,\"variations\":[{\"name\":\"Tomatensoße mit Hähnchenfleisch und Paprika\",\"allergens_and_add\":\"glutenhaltiges Getreide, Weizen\"},{\"name\":\"Tomatensoße mit Kräutern\",\"allergens_and_add\":\"glutenhaltiges Getreide, Weizen\"},{\"name\":\"Bolognese aus Tofu\",\"allergens_and_add\":\"glutenhaltiges Getreide, Soja, Sellerie, Weizen\"},{\"name\":\"Tomatensoße mit Schinken\",\"allergens_and_add\":\"Konservierungsstoff, Antioxidationsmittel, Phosphat, glutenhaltiges Getreide, Weizen\"},{\"name\":\"Käse-Tomatensoße \\\"Parmarosa\\\"\",\"allergens_and_add\":\"glutenhaltiges Getreide, Eier, Milch/ Milchzucker, Weizen\"}],\"price\":\"2,10 € / 4,60 € / 6,35 €\"}]},{\"meal_type\":\"WOK\",\"sub_meals\":[{\"name\":\"Asiatische Gemüsepfanne mit Mie Nudeln\",\"additional_ingredients\":[\"Like Chicken Nuggets\",\"Schweinefleisch in Kokosmilch\"],\"allergens\":\"Konservierungsstoff, glutenhaltiges Getreide, Soja, Weizen\",\"variations\":null,\"price\":\"3,50 € / 5,80 € / 7,40 €\"},{\"name\":\"Asiatische Gemüsepfanne mit Mie Nudeln\",\"additional_ingredients\":[],\"allergens\":\"Konservierungsstoff, glutenhaltiges Getreide, Soja, Weizen\",\"variations\":null,\"price\":\"2,50 € / 4,80 € / 6,60 €\"}]},{\"meal_type\":\"Gemüsebeilage\",\"sub_meals\":[{\"name\":\"Balkangemüse\",\"additional_ingredients\":[],\"allergens\":null,\"variations\":null,\"price\":\"0,55 € / 1,10 € / 1,50 €\"},{\"name\":\"Bohnengemüse\",\"additional_ingredients\":[],\"allergens\":null,\"variations\":null,\"price\":\"0,55 € / 1,10 € / 1,50 €\"}]},{\"meal_type\":\"Sättigungsbeilage\",\"sub_meals\":[{\"name\":\"Pommes frites\",\"additional_ingredients\":[],\"allergens\":null,\"variations\":null,\"price\":\"0,75 € / 1,35 € / 1,75 €\"},{\"name\":\"Dillkartoffeln\",\"additional_ingredients\":[],\"allergens\":null,\"variations\":null,\"price\":\"0,55 € / 1,10 € / 1,50 €\"}]}]}";
    // let new = serde_json::from_str::<types::CanteenMealsDay>(_new).unwrap();
    // let diff = stuwe_request_funcs::diff_canteen_meals(Some(old), &new);
    // println!("{:#?}", diff);
    // return;

    tokio::spawn(async {
        if let Err(e) = init_openmensa_canteenlist().await {
            log::error!("OpenMensa list fetch failed: {}", e);
        }
    });

    // always update cache on startup
    // dont pass 'today updated tx', this would cause erroneous WS broadcasts when cache
    // is too outdated or doesnt exist
    match update_cache(None).await {
        Ok(_) => log::info!("Cache updated!"),
        Err(e) => log::error!("Cache update failed: {}", e),
    }

    // set up broadcast channel to notify WS clients whenever today's canteen plans changed
    let (today_updated_tx, _) = broadcast::channel(20);

    start_canteen_cache_job(today_updated_tx.clone()).await;

    let listener = TcpListener::bind("0.0.0.0:9090")
        .await
        .expect("Unable to conne to connect to the server");

    log::info!("Listening on {}", listener.local_addr().unwrap());

    let app = routes::app(today_updated_tx).await;

    // used for building profiling data as i'm too lazy to set up test/bench
    // if env::var_os("PGOONLY").is_some() {
    //     for _ in 0..20 {
    //         update_cache().await.unwrap();
    //     }
    //     std::process::exit(0);
    // }

    axum::serve(listener, app)
        .await
        .expect("Error serving application");
}
