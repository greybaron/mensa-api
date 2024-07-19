use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, NaiveDate};
use scraper::{Element, ElementRef, Html, Selector};
use std::collections::BTreeMap;
use std::time::Instant;

use crate::db_operations::{get_jsonmeals_from_db, mensa_name_get_id_db, save_meal_to_db};
use crate::types::{DataForMensaForDay, MealGroup, SingleMeal};

pub async fn parse_and_save_meals(day: NaiveDate) -> Result<Vec<u8>> {
    let mut today_changed_mensen_ids = vec![];

    let date_string = build_date_string(day);

    // getting data from server
    let downloaded_html = reqwest_get_html_text(&date_string).await?;

    let all_data_for_day = extract_data_from_html(&downloaded_html, day).await?;
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

    log::info!("reqwest_get_html_text: {:.2?}", now.elapsed());
    Ok(txt)
}

async fn extract_data_from_html(
    html_text: &str,
    requested_date: NaiveDate,
) -> Result<Vec<DataForMensaForDay>> {
    let mut all_data_for_day = vec![];

    let now = Instant::now();
    let document = Html::parse_fragment(html_text);

    let date_button_group_sel = Selector::parse(r#"button.date-button.is--active"#).unwrap();
    let active_date_button = document
        .select(&date_button_group_sel)
        .next()
        .context("Recv. StuWe site is invalid (has no date)")?;

    let received_date_str = active_date_button.attr("data-date").unwrap().to_owned();
    let received_date = NaiveDate::parse_from_str(&received_date_str, "%Y-%m-%d")
        .context("unexpected StuWe date format")?;

    // if received date != requested date -> return empty meals struct (isn't an error, just StuWe being weird)
    if received_date != requested_date {
        return Ok(vec![]);
    }

    let container_sel = Selector::parse(r#"div.meal-overview"#).unwrap();

    let meal_containers: Vec<ElementRef> = document.select(&container_sel).collect();
    if meal_containers.is_empty() {
        return Err(anyhow!("StuWe site has no meal containers"));
    }

    for meal_container in meal_containers {
        if let Some(mensa_title_element) = meal_container.prev_sibling_element() {
            let mensa_title = mensa_title_element.inner_html();
            let meals = extract_mealgroup_from_htmlcontainer(meal_container)?;
            if let Some(mensa_id) = mensa_name_get_id_db(&mensa_title)? {
                all_data_for_day.push(DataForMensaForDay {
                    mensa_id,
                    meal_groups: meals,
                });
            } else {
                log::warn!("Mensa not found in DB: {}", mensa_title);
            }
        }
    }

    log::info!("HTML → Data: {:.2?}", now.elapsed());
    Ok(all_data_for_day)
}

fn extract_mealgroup_from_htmlcontainer(meal_container: ElementRef<'_>) -> Result<Vec<MealGroup>> {
    let mut v_meal_groups: Vec<MealGroup> = Vec::new();

    let meal_sel = Selector::parse(r#"div.type--meal"#).unwrap();
    let meal_type_sel = Selector::parse(r#"div.meal-tags>.tag"#).unwrap();
    let title_sel = Selector::parse(r#"h4"#).unwrap();
    let additional_ingredients_sel = Selector::parse(r#"div.meal-components"#).unwrap();
    let price_sel = Selector::parse(r#"div.meal-prices>span"#).unwrap();

    // quick && dirty
    for meal_element in meal_container.select(&meal_sel) {
        let meal_type = meal_element
            .select(&meal_type_sel)
            .next()
            .context("meal category element not found")?
            .inner_html();

        let title = meal_element
            .select(&title_sel)
            .next()
            .context("meal title element not found")?
            .inner_html()
            .replace("&nbsp;", " ")
            .replace("&amp;", "&");

        let additional_ingredients =
            if let Some(item) = meal_element.select(&additional_ingredients_sel).next() {
                let text = item.inner_html();
                // for whatever reason there might be, sometimes this element exists without any content
                if !text.is_empty() {
                    let mut add_ingr_dedup: Vec<String> = vec![];
                    let inner_html = item.inner_html();
                    let iter = inner_html.split('·').map(|slice| slice.trim().to_string());
                    for ingr in iter {
                        if !add_ingr_dedup.contains(&ingr) {
                            add_ingr_dedup.push(ingr);
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
        meal_element.select(&price_sel).for_each(|price_element| {
            price += &price_element
                .inner_html()
                .replace("&nbsp;", " ")
                .replace("&amp;", "&");
        });
        price = price.trim().to_string();

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
                    }],
                });
            }
            Some(meal_group) => {
                // meal group of this type already exists, add meal to it

                let add_meal = SingleMeal {
                    name: title,
                    price,
                    additional_ingredients,
                };

                if !meal_group.sub_meals.contains(&add_meal) {
                    meal_group.sub_meals.push(add_meal);
                }
            }
        }
    }

    Ok(v_meal_groups)
}

pub async fn get_mensen() -> Result<BTreeMap<u8, String>> {
    let mut mensen = BTreeMap::new();

    // pass invalid date to get empty page (dont need actual data) with all mensa locations
    let html_text = reqwest_get_html_text("a").await.unwrap_or_default();
    let document = Html::parse_fragment(&html_text);
    let mensa_list_sel = Selector::parse("#locations>li").unwrap();
    let mensa_item_sel = Selector::parse("span").unwrap();
    for list_item in document.select(&mensa_list_sel) {
        if let Some(mensa_id) = list_item.value().attr("data-location") {
            if let Ok(mensa_id) = mensa_id.parse::<u8>() {
                if let Some(mensa_name) = list_item.select(&mensa_item_sel).next() {
                    mensen.insert(mensa_id, mensa_name.inner_html());
                }
            }
        }
    }

    if mensen.is_empty() {
        log::warn!("Failed to load mensen from stuwe, falling back");
        Ok(BTreeMap::from(
            [
                (153, "Cafeteria Dittrichring"),
                (127, "Mensaria am Botanischen Garten"),
                (118, "Mensa Academica"),
                (106, "Mensa am Park"),
                (115, "Mensa am Elsterbecken"),
                (162, "Mensa am Medizincampus"),
                (111, "Mensa Peterssteinweg"),
                (140, "Mensa Schönauer Straße"),
                (170, "Mensa An den Tierklinik"),
            ]
            .map(|(id, name)| (id, name.to_string())),
        ))
    } else {
        Ok(mensen)
    }
}
