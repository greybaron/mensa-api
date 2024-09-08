use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, NaiveDate};
use lazy_static::lazy_static;
use scraper::{Element, ElementRef, Html, Selector};
use std::collections::BTreeMap;
use std::time::Instant;

use crate::constants::MENSEN_MAP_INV;
use crate::db_operations::{add_mensa_id_db, get_jsonmeals_from_db, save_meal_to_db};
use crate::types::{DataForMensaForDay, MealGroup, MealVariation, SingleMeal};

pub async fn _run_benchmark() {
    println!("downloading htmls");
    let today = chrono::Local::now();

    let mut strings: Vec<String> = Vec::new();
    for i in 0..7 {
        let day: chrono::DateTime<chrono::FixedOffset> = (today + chrono::Duration::days(i)).into();

        if ![chrono::Weekday::Sat, chrono::Weekday::Sun].contains(&day.weekday()) {
            strings.push(
                reqwest_get_html_text(&build_date_string(day.date_naive()))
                    .await
                    .unwrap(),
            );
        }
    }

    println!("got {} htmls", strings.len());
    let now = Instant::now();
    let its = 100;

    // ST
    for _ in 0..its {
        for string in &strings {
            extract_data_from_html(string).await.unwrap();
        }
    }

    println!("{} in {:.2?}", its * strings.len(), now.elapsed());
}

pub async fn parse_and_save_meals(day: NaiveDate) -> Result<Vec<u32>> {
    let mut today_changed_mensen_ids = vec![];

    let date_string = build_date_string(day);

    // getting data from server
    let downloaded_html = reqwest_get_html_text(&date_string).await?;

    let all_data_for_day = extract_data_from_html(&downloaded_html).await?;
    // serialize downloaded meals
    for mensa_data_for_day in all_data_for_day {
        let downloaded_json_text = serde_json::to_string(&mensa_data_for_day.meal_groups).unwrap();
        let db_json_text = get_jsonmeals_from_db(&date_string, mensa_data_for_day.mensa_id).await?;

        // if downloaded meals are different from cached meals, update cache
        if db_json_text.is_none() || downloaded_json_text != db_json_text.unwrap() {
            log::info!(
                "updating cache: Mensa={} Date={}",
                mensa_data_for_day.mensa_id,
                date_string
            );
            save_meal_to_db(
                &date_string,
                mensa_data_for_day.mensa_id,
                &downloaded_json_text,
            )
            .await?;

            if day.weekday() == chrono::Local::now().weekday() {
                today_changed_mensen_ids.push(mensa_data_for_day.mensa_id);
            }
        }
    }

    Ok(today_changed_mensen_ids)
}

pub fn build_date_string(requested_date: NaiveDate) -> String {
    let (year, month, day) = (
        requested_date.year(),
        requested_date.month(),
        requested_date.day(),
    );

    format!("{:04}-{:02}-{:02}", year, month, day)
}

async fn reqwest_get_html_text(date: &str) -> Result<String> {
    let now = Instant::now();
    let url_base =
        "https://www.studentenwerk-leipzig.de/mensen-cafeterien/speiseplan?date=".to_string();
    let txt = reqwest::get(url_base + date).await?.text().await?;

    log::debug!("reqwest_get_html_text: {:.2?}", now.elapsed());
    Ok(txt)
}

async fn extract_data_from_html(html_text: &str) -> Result<Vec<DataForMensaForDay>> {
    let mut all_data_for_day = vec![];

    let now = Instant::now();

    let document = Html::parse_fragment(html_text);

    lazy_static! {
        static ref DATE_BUTTON_GROUPSEL: Selector =
            Selector::parse(r#"button.date-button.is--active"#).unwrap();
        // static ref CONTAINER_SEL: Selector = Selector::parse(r#"div.meal-overview"#).unwrap();
        static ref TITLE_SEL: Selector = Selector::parse("h3").unwrap();

    };

    document
        .select(&DATE_BUTTON_GROUPSEL)
        .next()
        .context("Recv. StuWe site is invalid (has no date)")?;

    let title_elements = document.select(&TITLE_SEL);

    for mensa_name in title_elements {
        let mensa_title = mensa_name.inner_html();
        let meals = extract_mealgroup_from_htmlcontainer(
            mensa_name
                .next_sibling_element()
                .context("h3 without meal container")?,
        )?;
        let mensen = MENSEN_MAP_INV.read().unwrap();
        if let Some(mensa_id) = mensen.get(&mensa_title) {
            all_data_for_day.push(DataForMensaForDay {
                mensa_id: *mensa_id,
                meal_groups: meals,
            });
        } else {
            // drop the readguard to not deadlock write (if let Some() only drops after else)
            drop(mensen);
            log::warn!("Adding new Mensa to db: {}", mensa_title);

            if let Ok(mensa_id) = extract_mensaid(&document, &mensa_title) {
                add_mensa_id_db(mensa_id, &mensa_title).unwrap();
                MENSEN_MAP_INV
                    .write()
                    .unwrap()
                    .insert(mensa_title, mensa_id);
            }
        }
    }

    log::info!("HTML → Data: {:.2?}", now.elapsed());
    Ok(all_data_for_day)
}

fn extract_mealgroup_from_htmlcontainer(meal_container: ElementRef<'_>) -> Result<Vec<MealGroup>> {
    let mut v_meal_groups: Vec<MealGroup> = Vec::new();

    lazy_static! {
        static ref MEAL_SEL: Selector = Selector::parse(r#"div.type--meal"#).unwrap();
        static ref MEAL_TYPE_SEL: Selector = Selector::parse(r#"div.meal-tags>.tag"#).unwrap();
        static ref TITLE_SEL: Selector = Selector::parse(r#"h4"#).unwrap();
        static ref ADDITIONAL_INGREDIENTS_SEL: Selector =
            Selector::parse(r#"div.meal-components"#).unwrap();
        static ref PRICE_SEL: Selector = Selector::parse(r#"div.meal-prices>span"#).unwrap();
        static ref ALLERGENS_SEL: Selector = Selector::parse(r#"div.meal-allergens>p"#).unwrap();
        static ref VARIATIONS_SEL: Selector = Selector::parse(r#"div.meal-subitems"#).unwrap();
        static ref H5_SELECTOR: Selector = Selector::parse("h5").unwrap();
        static ref P_SELECTOR: Selector = Selector::parse("p").unwrap();
    };

    // quick && dirty
    for meal_element in meal_container.select(&MEAL_SEL) {
        let meal_type = meal_element
            .select(&MEAL_TYPE_SEL)
            .next()
            .context("meal category element not found")?
            .inner_html();

        let title = meal_element
            .select(&TITLE_SEL)
            .next()
            .context("meal title element not found")?
            .inner_html()
            .replace("&nbsp;", " ")
            .replace("&amp;", "&");

        let additional_ingredients =
            if let Some(item) = meal_element.select(&ADDITIONAL_INGREDIENTS_SEL).next() {
                let text = item.inner_html();
                // for whatever reason there might be, sometimes this element exists without any content
                if !text.is_empty() {
                    let mut add_ingr_dedup: Vec<String> = vec![];
                    let inner_html = item.inner_html();
                    let iter = inner_html.split('·').map(|slice| slice.trim().to_string());
                    for ingr in iter {
                        let clean = ingr.replace("&nbsp;", " ").replace("&amp; ", "");
                        if !add_ingr_dedup.contains(&clean) {
                            add_ingr_dedup.push(clean);
                        }
                    }

                    add_ingr_dedup
                } else {
                    // in that case, return empty vec (otherwise it would be a vec with one empty string in it)
                    vec![]
                }
                // Sosumi
            } else {
                vec![]
            };

        let mut price = String::new();
        meal_element.select(&PRICE_SEL).for_each(|price_element| {
            price += &price_element
                .inner_html()
                .replace("&nbsp;", " ")
                .replace("&amp;", "&");
        });
        price = price.trim().to_string();

        let allergens = meal_element
            .select(&ALLERGENS_SEL)
            .next()
            .map(|el| el.inner_html());

        let variations = meal_element.select(&VARIATIONS_SEL).next().map(|el| {
            let mut variations_vec: Vec<MealVariation> = vec![];

            for variation in el.child_elements() {
                let name = variation
                    .select(&H5_SELECTOR)
                    .next()
                    .unwrap()
                    .text()
                    .next()
                    .unwrap()
                    .trim()
                    .to_string();

                let allergens_and_add = variation
                    .select(&P_SELECTOR)
                    .next()
                    .map(|el| el.text().last().unwrap().replace(": ", "").to_string());

                variations_vec.push(MealVariation {
                    name,
                    allergens_and_add,
                });
            }

            variations_vec
        });

        // oh my
        // oh my
        match v_meal_groups
            .iter_mut()
            .find(|meal_group| meal_group.meal_type == meal_type)
        {
            None => {
                // doesn't exist yet, create new meal group of new meal type
                v_meal_groups.push(MealGroup {
                    meal_type,
                    sub_meals: vec![SingleMeal {
                        name: title,
                        price,
                        additional_ingredients,
                        allergens,
                        variations,
                    }],
                });
            }
            Some(meal_group) => {
                // meal group of this type already exists, add meal to it

                let add_meal = SingleMeal {
                    name: title,
                    price,
                    additional_ingredients,
                    allergens,
                    variations,
                };

                if !meal_group.sub_meals.contains(&add_meal) {
                    meal_group.sub_meals.push(add_meal);
                }
            }
        }
    }

    Ok(v_meal_groups)
}

// pub async fn get_mensen() -> Result<BTreeMap<u32, String>> {
//     let mut mensen = BTreeMap::new();

//     // pass invalid date to get empty page (dont need actual data) with all mensa locations
//     let html_text = reqwest_get_html_text("a").await.unwrap_or_default();
//     let document = Html::parse_fragment(&html_text);
//     let mensa_list_sel = Selector::parse("#locations>li").unwrap();
//     let mensa_item_sel = Selector::parse("span").unwrap();
//     for list_item in document.select(&mensa_list_sel) {
//         if let Some(mensa_id) = list_item.value().attr("data-location") {
//             if let Ok(mensa_id) = mensa_id.parse::<u32>() {
//                 if let Some(mensa_name) = list_item.select(&mensa_item_sel).next() {
//                     mensen.insert(mensa_id, mensa_name.inner_html());
//                 }
//             }
//         }
//     }

//     if mensen.is_empty() {
//         log::warn!("Failed to load mensen from stuwe, falling back");
//         Ok(BTreeMap::from(
//             [
//                 (153, "Cafeteria Dittrichring"),
//                 (127, "Mensaria am Botanischen Garten"),
//                 (118, "Mensa Academica"),
//                 (106, "Mensa am Park"),
//                 (115, "Mensa am Elsterbecken"),
//                 (162, "Mensa am Medizincampus"),
//                 (111, "Mensa Peterssteinweg"),
//                 (140, "Mensa Schönauer Straße"),
//                 (170, "Mensa An den Tierklinik"),
//             ]
//             .map(|(id, name)| (id, name.to_string())),
//         ))
//     } else {
//         Ok(mensen)
//     }
// }

fn extract_mensaid(document: &Html, mensa_title: &str) -> Result<u32> {
    // let mut mensen = BTreeMap::new();

    // pass invalid date to get empty page (dont need actual data) with all mensa locations
    // let html_text = reqwest_get_html_text("a").await.unwrap_or_default();
    // let document = Html::parse_fragment(&html_text);
    lazy_static! {
        static ref MENSA_LIST_SEL: Selector = Selector::parse("#locations>li").unwrap();
        static ref MENSA_ITEM_SEL: Selector = Selector::parse("span").unwrap();
    };

    let mensa_li = document
        .select(&MENSA_LIST_SEL)
        .find(|li| li.first_element_child().unwrap().inner_html() == mensa_title);
    if let Some(mensa_li) = mensa_li {
        if let Some(mensa_id) = mensa_li.value().attr("data-location") {
            if let Ok(mensa_id) = mensa_id.parse::<u32>() {
                return Ok(mensa_id);
            }
        }
    }

    // for list_item in document.select(&MENSA_LIST_SEL) {
    //     if let Some(mensa_id) = list_item.value().attr("data-location") {
    //         if let Ok(mensa_id) = mensa_id.parse::<u32>() {
    //             if let Some(mensa_name) = list_item.select(&MENSA_ITEM_SEL).next() {
    //                 mensen.insert(mensa_id, mensa_name.inner_html());
    //             }
    //         }
    //     }
    // }

    Err(anyhow!("Failed to extract mensa id"))

    // if mensen.is_empty() {
    //     log::warn!("Failed to load mensen from stuwe, falling back");
    //     Ok(BTreeMap::from(
    //         [
    //             (153, "Cafeteria Dittrichring"),
    //             (127, "Mensaria am Botanischen Garten"),
    //             (118, "Mensa Academica"),
    //             (106, "Mensa am Park"),
    //             (115, "Mensa am Elsterbecken"),
    //             (162, "Mensa am Medizincampus"),
    //             (111, "Mensa Peterssteinweg"),
    //             (140, "Mensa Schönauer Straße"),
    //             (170, "Mensa An den Tierklinik"),
    //         ]
    //         .map(|(id, name)| (id, name.to_string())),
    //     ))
    // } else {
    //     Ok(mensen)
    // }
}

pub fn invert_map<K, V>(map: &BTreeMap<K, V>) -> BTreeMap<V, K>
where
    K: Clone + Ord,
    V: Clone + Ord,
{
    map.iter().map(|(k, v)| (v.clone(), k.clone())).collect()
}
