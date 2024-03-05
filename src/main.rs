use serde::{Deserialize, Serialize};
use std::io;
use serde_json::Result;
use std::env;
use std::collections::HashSet;
use term_size::dimensions_stdout;

#[derive(Debug, Deserialize, Serialize)]
struct Reward {
    rarity: String,
    chance: f64,
    item: RewardItem,
}

#[derive(Debug, Deserialize, Serialize)]
struct RewardItem {
    name: String,
    uniqueName: String,
    #[serde(default)]
    warframeMarket: Option<WarframeMarket>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WarframeMarket {
    id: String,
    urlName: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Patchlog {
    name: String,
    date: String,
    url: String,
    additions: String,
    changes: String,
    fixes: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Component {
    name: String,
    uniqueName: String,
    description: Option<String>,
    #[serde(rename = "type")]
    type_: Option<String>,
    tradable: bool,
    category: Option<String>,
    productCategory: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Introduced {
    name: String,
    url: String,
    aliases: Vec<String>,
    parent: String,
    date: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Item {
    name: String,
    uniqueName: String,
    description: Option<String>,
    #[serde(rename = "type")]
    type_: String,
    tradable: bool,
    category: Option<String>,
    productCategory: Option<String>,
    patchlogs: Option<Vec<Patchlog>>,
    components: Option<Vec<Component>>,
    introduced: Option<Introduced>,
    estimatedVaultDate: Option<String>,
    rewards: Option<Vec<Reward>>, // Include the rewards property
}

#[derive(Debug, PartialEq)]
enum RelicType {
    Lith,
    Meso,
    Neo,
    Axi,
}

impl RelicType {
    fn from_str(s: &str) -> Option<RelicType> {
        match s.to_lowercase().as_str() {
            "lith" => Some(RelicType::Lith),
            "meso" => Some(RelicType::Meso),
            "neo" => Some(RelicType::Neo),
            "axi" => Some(RelicType::Axi),
            _ => None,
        }
    }
} 

fn str_is_valid_relic_of_type(s: &str, relic_type: &RelicType) -> bool {
    let s_lowercase = s.to_lowercase();
    match relic_type {
        RelicType::Lith => s_lowercase.starts_with("lith"),
        RelicType::Meso => s_lowercase.starts_with("meso"),
        RelicType::Neo => s_lowercase.starts_with("neo"),
        RelicType::Axi => s_lowercase.starts_with("axi"),
    }
}

#[derive(Debug, PartialEq)]
enum OutputFormat {
    Default,
    Search,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Default
    }
}

impl Item {
    fn get_relic_short_name(&self) -> String {
        let segments: Vec<&str> = self.name.split_whitespace().take(2).collect();
        segments.join(" ")
    }
}

fn wrap_text(text: &str, prefix: &str, max_width: usize, indent_after_first: usize) -> Vec<String> {
    let mut lines = vec![];
    let mut current_line = String::from(prefix);
    let prefix_width = unicode_width::UnicodeWidthStr::width(prefix);

    for word in text.split_whitespace() {
        let word_width = unicode_width::UnicodeWidthStr::width(word);
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if unicode_width::UnicodeWidthStr::width(format!("{} {} {}", prefix, &current_line, word).as_str()) <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line.clone());
            current_line = " ".repeat(indent_after_first);
            current_line.push_str(word);
        }
    }
    lines.push(current_line);

    lines
}

fn log_items(items: Vec<Item>, output_format: OutputFormat, has_relic_arg: bool) {
    let mut unique_items: HashSet<String> = HashSet::new();


    let (term_width, _) = dimensions_stdout().unwrap_or((80, 24)); // Default width: 80, height: 24

    for item in items {
        match output_format {
            OutputFormat::Default => {
                // Calculate border width
                let border_width = term_width - 2; // Subtract 2 for the borders
                
                println!("┌{}┐", "─".repeat(border_width));
                println!("│ Name: {}", item.name);
                println!("│ UniqueName: {}", item.uniqueName);
                if let Some(description) = &item.description {
                    let desc_lines = wrap_text(description, "Description:", border_width, 2);
                    for line in desc_lines {
                        println!("│ {}", line);
                    }
                }
                println!("│ Type: {}", item.type_);
                println!("│ Tradable: {}", item.tradable);
                if let Some(category) = &item.category {
                    println!("│ Category: {}", category);
                }
                if let Some(product_category) = &item.productCategory {
                    println!("│ Product Category: {}", product_category);
                }
                if let Some(introduced) = &item.introduced {
                    println!("│ Introduced Date: {}", introduced.date);
                }
                if let Some(vault_date) = &item.estimatedVaultDate {
                    println!("│ Estimated Vault Date: {}", vault_date);
                }
                if let Some(rewards) = &item.rewards {
                    for reward in rewards {
                        println!("│   - {}", reward.item.name);
                    }
                }
                println!("└{}┘", "─".repeat(border_width));
            }
            // OutputFormat::Default => {
            //     println!("Name: {}", item.name);
            //     println!("UniqueName: {}", item.uniqueName);
            //     println!("Description: {:?}", item.description);
            //     println!("Type: {}", item.type_);
            //     println!("Tradable: {}", item.tradable);
            //     println!("Category: {:?}", item.category);
            //     println!("Product Category: {:?}", item.productCategory);
            //     println!("Introduced Date: {:?}", item.introduced.as_ref().map(|v| &v.date));
            //     println!("Estimated Vault Date: {:?}", item.estimatedVaultDate);

            //     // Print rewards if available
            //     if let Some(rewards) = &item.rewards {
            //         for reward in rewards {
            //             println!("  - {}", reward.item.name);
            //         }
            //     }

            //     println!("---");
            // }
            OutputFormat::Search => {
                if has_relic_arg {
                    let short_name = item.get_relic_short_name();
                    if unique_items.insert(short_name.clone()) {
                        println!("{}", short_name);
                    }
                } else {
                    println!("{}", item.name);
                }
            }
        }
    }
}

fn filter_items_by_relic_type(items: Vec<Item>, relic_type: Option<RelicType>) -> Vec<Item> {
    items.into_iter().filter(|item| {
        // Filter logic: check if the item's type is "relic"
        let is_relic = &item.type_ == "Relic";

        // println!("type: {:?}, is_relic: {:?}", item.type_, is_relic);
        // If a relic type was provided, additionally check if the item's uniqueName starts with the string form of the relic type
        let matches_relic_type = match &relic_type {
            Some(relic_type) => {
                str_is_valid_relic_of_type(&item.uniqueName, &relic_type)
            },
            None => true, // If no relic type was provided, always consider it a match
        };

        // Return true if both conditions are met
        is_relic && matches_relic_type
    }).collect()
}

fn filter_items_by_search_term(items: Vec<Item>, search_term: Option<String>) -> Vec<Item> {
    match search_term {
        Some(term) => {
            let term_lowercase = term.to_lowercase();
            items.into_iter().filter(|item| {
                item.name.to_lowercase().starts_with(&term_lowercase) ||
                item.uniqueName.to_lowercase().starts_with(&term_lowercase)
            }).collect()
        },
        None => items,
    }
}
// get_wf_items() { cat ./data.json | ./target/release/wf_api_quick --log-items "$@" }
// search_relics () { get_wf_items --search "$(get_wf_items --fmt:search --relic | fzf)" }     
// search_wf_items () { get_wf_items --search "$(get_wf_items --fmt:search | fzf)" }     

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Check if "--relic" argument is passed and get the relic type if provided
    let relic_index = args.iter().position(|arg| arg == "--relic");
    let relic_type = relic_index
        .and_then(|index| args.get(index + 1))
        .and_then(|s| RelicType::from_str(s));
    let has_relic_arg = relic_index.is_some();

    // Check if "--search" argument is passed and get the search term if provided
    let search_index = args.iter().position(|arg| arg == "--search");
    let search_term = search_index
        .and_then(|index| args.get(index + 1))
        .cloned();

    // Read JSON data from stdin
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();

    let items: Vec<Item> = serde_json::from_str(&buffer)?;

    // Filter items by relic type if provided
    let filtered_items = if has_relic_arg {
        filter_items_by_relic_type(items, relic_type)
    } else {
        items
    };

    // Filter items by search term if provided
    let filtered_items = if let Some(term) = search_term {
        filter_items_by_search_term(filtered_items, Some(term))
    } else {
        filtered_items
    };

    // Check if "--fmt:search" argument is passed
    let output_format = if args.contains(&String::from("--fmt:search")) {
        OutputFormat::Search
    } else {
        OutputFormat::Default
    };

    // Check if "--log-items" argument is passed
    if args.contains(&String::from("--log-items")) {
        log_items(filtered_items, output_format, has_relic_arg);
    }

    Ok(())
}

