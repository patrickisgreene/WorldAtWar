use serde::{Deserialize, Serialize};
use std::fs;
use std::error::Error;
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
struct Coordinates {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct City {
    name: String,
    coordinates: Coordinates,
}

#[derive(Debug, Serialize, Deserialize)]
struct CountryData {
    name: String,
    code: String,
    population: u64,
    capital: usize, // Index into cities list
    cities: Vec<City>,
}

#[derive(Debug, Deserialize)]
struct RestCountryResponse {
    name: RestCountryName,
    #[serde(rename = "cca2")]
    code: String,
    population: u64,
    capital: Option<Vec<String>>,
    #[serde(rename = "capitalInfo")]
    capital_info: Option<CapitalInfo>,
}

#[derive(Debug, Deserialize)]
struct RestCountryName {
    common: String,
}

#[derive(Debug, Deserialize)]
struct CapitalInfo {
    latlng: Option<Vec<f64>>,
}

#[derive(Debug, Deserialize)]
struct GeoDBResponse {
    data: Vec<GeoDBCity>,
}

#[derive(Debug, Deserialize)]
struct GeoDBCity {
    name: String,
    latitude: f64,
    longitude: f64,
}

fn get_existing_country_codes() -> Result<HashSet<String>, Box<dyn Error>> {
    let mut existing = HashSet::new();

    // Read assets/countries directory for .country.ron files
    let assets_dir = format!("{}/../../../assets/countries", env!("CARGO_MANIFEST_DIR"));

    if let Ok(entries) = fs::read_dir(&assets_dir) {
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str() {
                // Check if it matches the pattern XX.country.ron
                if filename.ends_with(".country.ron") {
                    let country_code = filename.trim_end_matches(".country.ron");
                    existing.insert(country_code.to_string());
                }
            }
        }
    }

    Ok(existing)
}

async fn fetch_countries() -> Result<Vec<RestCountryResponse>, Box<dyn Error>> {
    // API now requires fields parameter - specify the fields we need
    let url = "https://restcountries.com/v3.1/all?fields=name,cca2,population,capital,capitalInfo";
    let response = reqwest::get(url).await?;
    let countries: Vec<RestCountryResponse> = response.json().await?;
    Ok(countries)
}

async fn fetch_cities(country_code: &str) -> Result<Vec<City>, Box<dyn Error>> {
    // Using GeoDB free service - no API key required!
    // Free tier allows max 10 results per request, so we'll paginate to get 50
    let mut all_cities = Vec::new();

    // Fetch 5 pages of 10 cities each (50 total)
    for page in 0..5 {
        let offset = page * 10;
        let url = format!(
            "http://geodb-free-service.wirefreethought.com/v1/geo/countries/{}/places?types=CITY&limit=10&offset={}&sort=-population",
            country_code, offset
        );

        let response = reqwest::get(&url).await?;

        // Check if we got a successful response
        if !response.status().is_success() {
            eprintln!("  Warning: API returned status {} for page {}", response.status(), page);
            break;
        }

        let geo_response: GeoDBResponse = response.json().await?;

        // If we got no results, stop paginating
        if geo_response.data.is_empty() {
            if page == 0 {
                eprintln!("  Warning: No cities found for {}", country_code);
            }
            break;
        }

        let page_count = geo_response.data.len();

        // Add cities from this page
        for city in geo_response.data {
            all_cities.push(City {
                name: city.name,
                coordinates: Coordinates {
                    lat: city.latitude,
                    lon: city.longitude,
                },
            });
        }

        if page == 0 {
            eprintln!("  Fetched {} cities (page 1/5)", page_count);
        }

        // Small delay between pagination requests to be respectful
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    if !all_cities.is_empty() {
        eprintln!("  Total cities fetched: {}", all_cities.len());
    }

    Ok(all_cities)
}

async fn process_country(country: &RestCountryResponse) -> Result<(), Box<dyn Error>> {
    let capital_name = country.capital
        .as_ref()
        .and_then(|caps| caps.first())
        .map(|s| s.as_str())
        .unwrap_or("");
    
    let capital_coords = country.capital_info
        .as_ref()
        .and_then(|info| info.latlng.as_ref())
        .and_then(|coords| {
            if coords.len() >= 2 {
                Some(Coordinates {
                    lat: coords[0],
                    lon: coords[1],
                })
            } else {
                None
            }
        });
    
    // Fetch cities for this country
    let mut cities = fetch_cities(&country.code).await.unwrap_or_default();
    
    // If no cities found or capital not in list, add capital manually
    let capital_index = if !capital_name.is_empty() {
        let cap_idx = cities.iter().position(|c| c.name.to_lowercase() == capital_name.to_lowercase());

        if let Some(idx) = cap_idx {
            idx
        } else {
            // Add capital to the beginning of the list
            cities.insert(0, City {
                name: capital_name.to_string(),
                coordinates: capital_coords.unwrap_or(Coordinates { lat: 0.0, lon: 0.0 }),
            });
            0
        }
    } else if !cities.is_empty() {
        0 // Default to first city if no capital specified
    } else {
        // No capital and no cities - add a placeholder
        cities.push(City {
            name: "Unknown".to_string(),
            coordinates: Coordinates { lat: 0.0, lon: 0.0 },
        });
        0
    };
    
    let country_data = CountryData {
        name: country.name.common.clone(),
        code: country.code.clone(),
        population: country.population,
        capital: capital_index,
        cities,
    };
    
    // Serialize to RON format
    let ron_string = ron::ser::to_string_pretty(&country_data, ron::ser::PrettyConfig::default())?;
    
    // Write to file
    let filename = format!("{}/../../../assets/countries/{}.country.ron", env!("CARGO_MANIFEST_DIR"), country.code);
    fs::write(&filename, ron_string)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Fetching country data...");

    let countries = fetch_countries().await?;
    println!("Found {} countries from API", countries.len());

    // Check which countries already have files
    let existing = get_existing_country_codes()?;
    println!("Found {} existing country files", existing.len());

    // Filter to only countries that need processing
    let countries_to_process: Vec<_> = countries
        .iter()
        .filter(|country| !existing.contains(&country.code))
        .collect();

    println!("Need to process {} new countries", countries_to_process.len());

    if countries_to_process.is_empty() {
        println!("All countries already processed!");
        return Ok(());
    }

    // Process only new countries
    for (idx, country) in countries_to_process.iter().enumerate() {
        print!("[{}/{}] Processing {}... ", idx + 1, countries_to_process.len(), country.code);
        match process_country(country).await {
            Ok(_) => println!("✓"),
            Err(e) => println!("✗ Error: {}", e),
        }

        // Add delay to be respectful of the free API
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    println!("Done!");

    Ok(())
}