use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[derive(Clone, Serialize, Deserialize)]
struct Product {
    name: String,
    quantity: f64,
    unit: String,
    minimum: f64,
    category: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct ShoppingItem {
    name: String,
    quantity: f64,
    unit: String,
    checked: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct MealPlanEntry {
    day: String,
    meal: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Chore {
    name: String,
    frequency: String,
    completed: bool,
    #[serde(default)]
    assigned_to: Option<String>,
    #[serde(default)]
    reminder_date: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct HouseholdMember {
    name: String,
    email: String,
}

#[derive(Serialize, Deserialize)]
struct HouseholdData {
    products: Vec<Product>,
    shopping: Vec<ShoppingItem>,
    #[serde(default)]
    meal_plan: Vec<MealPlanEntry>,
    #[serde(default)]
    chores: Vec<Chore>,
    #[serde(default)]
    members: Vec<HouseholdMember>,
    #[serde(default = "sample_recipes")]
    recipes: Vec<Recipe>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Recipe {
    name: String,
    note: String,
    ingredients: Vec<(String, f64, String)>,
}

fn sample_recipes() -> Vec<Recipe> {
    vec![
        Recipe {
            name: "Warm grain bowl".into(),
            note: "A simple pantry-friendly lunch".into(),
            ingredients: vec![("Oats".into(), 1.0, "cup".into()), ("Apples".into(), 1.0, "piece".into()), ("Honey".into(), 1.0, "tbsp".into())],
        },
        Recipe {
            name: "Bean and rice plate".into(),
            note: "A filling weeknight staple".into(),
            ingredients: vec![("Rice".into(), 1.0, "cup".into()), ("Canned beans".into(), 1.0, "can".into()), ("Tomatoes".into(), 2.0, "piece".into())],
        },
        Recipe {
            name: "Garden pasta".into(),
            note: "Quick pasta with a bright tomato sauce".into(),
            ingredients: vec![("Pasta".into(), 2.0, "portion".into()), ("Tomatoes".into(), 3.0, "piece".into()), ("Olive oil".into(), 1.0, "tbsp".into())],
        },
    ]
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Overview,
    Pantry,
    Shopping,
    Recipes,
    MealPlan,
    Chores,
}

impl Section {
    fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Pantry => "Stock overview",
            Self::Shopping => "Shopping list",
            Self::Recipes => "Recipes",
            Self::MealPlan => "Meal plan",
            Self::Chores => "Chores",
        }
    }
}

struct PantryDesk {
    data: HouseholdData,
    section: Section,
    search: String,
    new_product: String,
    new_quantity: String,
    new_shopping: String,
    new_shopping_quantity: String,
    new_meal: String,
    selected_day: String,
    new_chore: String,
    selected_frequency: String,
    selected_assignee: String,
    new_chore_reminder_date: String,
    new_member_name: String,
    new_member_email: String,
    recipe_name: String,
    recipe_note: String,
    recipe_ingredients: String,
    editing_recipe: Option<usize>,
    editing_product: Option<(usize, Product)>,
    pending_product_removal: Option<usize>,
    notice: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    data_path: PathBuf,
    screenshot_path: Option<PathBuf>,
    screenshot_requested: bool,
    frames: u32,
}

impl PantryDesk {
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    fn new(data_path: PathBuf, screenshot_path: Option<PathBuf>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let data = web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item("pantry-desk-household").ok().flatten())
            .and_then(|saved| serde_json::from_str(&saved).ok())
            .unwrap_or_else(Self::sample_data);
        #[cfg(not(target_arch = "wasm32"))]
        let data = match std::fs::read(&data_path) {
            Ok(bytes) => match serde_json::from_slice(&bytes) {
                Ok(data) => data,
                Err(error) => {
                    eprintln!("Could not read saved household data: {error}");
                    Self::sample_data()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Self::sample_data(),
            Err(error) => {
                eprintln!("Could not open saved household data: {error}");
                Self::sample_data()
            }
        };
        Self {
            data,
            section: Section::Overview,
            search: String::new(),
            new_product: String::new(),
            new_quantity: "1".to_owned(),
            new_shopping: String::new(),
            new_shopping_quantity: "1".to_owned(),
            new_meal: String::new(),
            selected_day: "Monday".to_owned(),
            new_chore: String::new(),
            selected_frequency: "Weekly".to_owned(),
            selected_assignee: String::new(),
            new_chore_reminder_date: String::new(),
            new_member_name: String::new(),
            new_member_email: String::new(),
            recipe_name: String::new(),
            recipe_note: String::new(),
            recipe_ingredients: String::new(),
            editing_recipe: None,
            editing_product: None,
            pending_product_removal: None,
            notice: None,
            #[cfg(not(target_arch = "wasm32"))]
            data_path,
            screenshot_path,
            screenshot_requested: false,
            frames: 0,
        }
    }

    fn sample_data() -> HouseholdData {
        HouseholdData {
            products: vec![
                Product { name: "Oats".into(), quantity: 2.0, unit: "bag".into(), minimum: 1.0, category: "Breakfast".into() },
                Product { name: "Rice".into(), quantity: 1.0, unit: "bag".into(), minimum: 1.0, category: "Grains".into() },
                Product { name: "Canned beans".into(), quantity: 4.0, unit: "can".into(), minimum: 2.0, category: "Pantry".into() },
                Product { name: "Apples".into(), quantity: 1.0, unit: "piece".into(), minimum: 3.0, category: "Produce".into() },
                Product { name: "Tomatoes".into(), quantity: 2.0, unit: "piece".into(), minimum: 3.0, category: "Produce".into() },
                Product { name: "Olive oil".into(), quantity: 1.0, unit: "bottle".into(), minimum: 1.0, category: "Pantry".into() },
            ],
            shopping: vec![
                ShoppingItem { name: "Bread".into(), quantity: 1.0, unit: "loaf".into(), checked: false },
                ShoppingItem { name: "Milk".into(), quantity: 2.0, unit: "carton".into(), checked: false },
                ShoppingItem { name: "Tea".into(), quantity: 1.0, unit: "box".into(), checked: true },
            ],
            meal_plan: vec![
                MealPlanEntry { day: "Monday".into(), meal: "Warm grain bowl".into() },
                MealPlanEntry { day: "Tuesday".into(), meal: "Bean and rice plate".into() },
            ],
            chores: vec![
                Chore { name: "Water the plants".into(), frequency: "Weekly".into(), completed: false, assigned_to: None, reminder_date: None },
                Chore { name: "Wipe kitchen surfaces".into(), frequency: "Daily".into(), completed: false, assigned_to: None, reminder_date: None },
            ],
            members: Vec::new(),
            recipes: sample_recipes(),
        }
    }

    fn persist(&mut self) {
        #[cfg(target_arch = "wasm32")]
        let result = (|| -> Result<(), String> {
            let window = web_sys::window().ok_or_else(|| "browser window is unavailable".to_owned())?;
            let storage = window.local_storage().map_err(|error| format!("could not access browser storage: {error:?}"))?
                .ok_or_else(|| "browser storage is unavailable".to_owned())?;
            let value = serde_json::to_string(&self.data).map_err(|error| error.to_string())?;
            storage.set_item("pantry-desk-household", &value).map_err(|error| format!("could not save browser data: {error:?}"))
        })();
        #[cfg(not(target_arch = "wasm32"))]
        let result = (|| -> Result<(), String> {
            if let Some(parent) = self.data_path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            let bytes = serde_json::to_vec_pretty(&self.data).map_err(|error| error.to_string())?;
            std::fs::write(&self.data_path, bytes).map_err(|error| error.to_string())
        })();
        self.notice = Some(match result {
            Ok(()) => "Saved".to_owned(),
            Err(error) => format!("Could not save changes: {error}"),
        });
    }

    fn low_stock_count(&self) -> usize {
        self.data.products.iter().filter(|product| product.quantity <= product.minimum).count()
    }

    fn add_shopping(&mut self, name: String, quantity: f64, unit: String) {
        let name = name.trim();
        if name.is_empty() {
            self.notice = Some("Enter an item name first.".into());
            return;
        }
        if let Some(item) = self.data.shopping.iter_mut().find(|item| item.name.eq_ignore_ascii_case(name) && !item.checked) {
            item.quantity += quantity;
        } else {
            self.data.shopping.push(ShoppingItem { name: name.into(), quantity, unit, checked: false });
        }
        self.persist();
    }

    fn add_missing_stock_to_shopping(&mut self) {
        let missing: Vec<ShoppingItem> = self.data.products.iter().filter_map(|product| {
            let amount = missing_stock_amount(product.quantity, product.minimum);
            (amount > 0.0).then(|| ShoppingItem { name: product.name.clone(), quantity: amount, unit: product.unit.clone(), checked: false })
        }).collect();
        if missing.is_empty() {
            self.notice = Some("All stock levels are at or above their minimum.".into());
            return;
        }
        for item in missing {
            if let Some(existing) = self.data.shopping.iter_mut().find(|entry| entry.name.eq_ignore_ascii_case(&item.name)) {
                existing.quantity = existing.quantity.max(item.quantity);
                existing.checked = false;
            } else {
                self.data.shopping.push(item);
            }
        }
        self.persist();
        self.notice = Some("Missing stock added to your shopping list.".into());
    }

    fn add_missing_ingredients(&mut self, recipe: &Recipe) {
        let missing: Vec<ShoppingItem> = recipe.ingredients.iter().filter_map(|(name, needed, unit)| {
            let available = self.data.products.iter()
                .find(|product| product.name.eq_ignore_ascii_case(name))
                .map_or(0.0, |product| product.quantity);
            let amount = missing_stock_amount(available, *needed);
            (amount > 0.0).then(|| ShoppingItem { name: name.clone(), quantity: amount, unit: unit.clone(), checked: false })
        }).collect();
        if missing.is_empty() {
            self.notice = Some("You have everything for this recipe.".into());
            return;
        }
        for item in missing {
            self.add_shopping(item.name, item.quantity, item.unit);
        }
        self.notice = Some("Missing ingredients added to your shopping list.".into());
    }

    fn show_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigation")
            .exact_width(206.0)
            .frame(egui::Frame::side_top_panel(&ctx.style()).fill(egui::Color32::from_rgb(27, 45, 39)).inner_margin(egui::Margin::same(18)))
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("HOME STOCK").size(14.0).strong().color(egui::Color32::from_rgb(172, 213, 184)));
                ui.add_space(28.0);
                for section in [Section::Overview, Section::Pantry, Section::Shopping, Section::Recipes, Section::MealPlan, Section::Chores] {
                    let selected = self.section == section;
                    let button = egui::Button::new(egui::RichText::new(section.title()).size(14.0).color(if selected { egui::Color32::WHITE } else { egui::Color32::from_rgb(195, 205, 198) }))
                        .fill(if selected { egui::Color32::from_rgb(51, 82, 68) } else { egui::Color32::TRANSPARENT })
                        .corner_radius(8.0)
                        .min_size(egui::vec2(168.0, 40.0));
                    if ui.add(button).clicked() {
                        self.section = section;
                    }
                    ui.add_space(4.0);
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(egui::RichText::new("Your household, at a glance").size(11.0).color(egui::Color32::from_rgb(149, 169, 156)));
                });
            });
    }

    fn show_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(self.section.title()).size(26.0).strong().color(egui::Color32::from_rgb(32, 49, 41)));
                ui.label(egui::RichText::new("A calm little space for your home essentials").size(13.0).color(egui::Color32::from_rgb(112, 124, 114)));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("LOCAL WORKSPACE").size(10.0).strong().color(egui::Color32::from_rgb(71, 120, 88)));
            });
        });
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(18.0);
    }

    fn card(ui: &mut egui::Ui, title: &str, value: impl ToString, subtitle: &str, color: egui::Color32) {
        egui::Frame::group(ui.style())
            .fill(egui::Color32::WHITE)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(230, 233, 228)))
            .corner_radius(12.0)
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.set_width(170.0);
                ui.set_min_height(104.0);
                ui.label(egui::RichText::new(title).size(12.0).strong().color(egui::Color32::from_rgb(72, 83, 74)));
                ui.add_space(8.0);
                ui.label(egui::RichText::new(value.to_string()).size(27.0).strong().color(color));
                ui.label(egui::RichText::new(subtitle).size(11.0).color(egui::Color32::from_rgb(82, 94, 84)));
            });
    }

    fn show_overview(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            Self::card(ui, "STOCK ITEMS", self.data.products.len(), "things you keep stocked", egui::Color32::from_rgb(50, 93, 67));
            Self::card(ui, "NEED A TOP-UP", self.low_stock_count(), "at or below your minimum", egui::Color32::from_rgb(181, 115, 49));
            Self::card(ui, "TO PICK UP", self.data.shopping.iter().filter(|item| !item.checked).count(), "on your shopping list", egui::Color32::from_rgb(68, 104, 151));
            Self::card(ui, "MEALS PLANNED", self.data.meal_plan.len(), "in this week's plan", egui::Color32::from_rgb(114, 91, 147));
            Self::card(ui, "CHORES TO DO", self.data.chores.iter().filter(|chore| !chore.completed).count(), "household tasks", egui::Color32::from_rgb(162, 108, 72));
        });
        ui.add_space(24.0);
        ui.label(egui::RichText::new("Needs a little attention").size(18.0).strong().color(egui::Color32::from_rgb(39, 54, 43)));
        ui.add_space(3.0);
        ui.label(egui::RichText::new("Items at or below your minimum stock").size(13.0).color(egui::Color32::from_rgb(75, 88, 77)));
        ui.add_space(12.0);
        let low: Vec<Product> = self.data.products.iter().filter(|product| product.quantity <= product.minimum).cloned().collect();
        if low.is_empty() {
            ui.label("Everything is above its minimum level.");
        } else {
            for product in low {
                egui::Frame::group(ui.style())
                    .fill(egui::Color32::from_rgb(255, 255, 255))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(224, 230, 222)))
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(12, 9))
                    .show(ui, |ui| {
                        let width = ui.available_width();
                        ui.horizontal(|ui| {
                            ui.add_sized([width * 0.45, 24.0], egui::Label::new(egui::RichText::new(&product.name).strong().color(egui::Color32::from_rgb(39, 54, 43))).truncate());
                            ui.add_sized([width * 0.32, 24.0], egui::Label::new(egui::RichText::new(format!("{} {} left", format_amount(product.quantity), product.unit)).color(egui::Color32::from_rgb(55, 67, 57))));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(format!("Min. {}", format_amount(product.minimum))).size(12.0).strong().color(egui::Color32::from_rgb(126, 76, 25)));
                            });
                        });
                    });
                ui.add_space(5.0);
            }
        }
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Coming up").size(18.0).strong().color(egui::Color32::from_rgb(39, 54, 43)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Open shopping list").clicked() { self.section = Section::Shopping; }
            });
        });
        ui.add_space(8.0);
        for item in self.data.shopping.iter().filter(|item| !item.checked).take(4) {
            ui.label(format!("•  {}  —  {} {}", item.name, format_amount(item.quantity), item.unit));
        }
    }

    fn show_product_editor(&mut self, ctx: &egui::Context) {
        let Some((index, mut product)) = self.editing_product.take() else { return; };
        let mut save = false;
        let mut cancel = false;
        let mut open = true;
        egui::Window::new("Edit stock item")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("Update the details for this stock item.");
                ui.add_space(8.0);
                ui.label("Name");
                ui.text_edit_singleline(&mut product.name);
                ui.label("Category");
                ui.text_edit_singleline(&mut product.category);
                ui.label("Unit");
                ui.text_edit_singleline(&mut product.unit);
                ui.horizontal(|ui| {
                    ui.label("Amount");
                    ui.add(egui::DragValue::new(&mut product.quantity).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Minimum stock");
                    ui.add(egui::DragValue::new(&mut product.minimum).speed(0.1));
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Save changes").clicked() { save = true; }
                    if ui.button("Cancel").clicked() { cancel = true; }
                });
            });
        if cancel || !open { return; }
        if save {
            if product.name.trim().is_empty() || product.category.trim().is_empty() || product.unit.trim().is_empty() {
                self.notice = Some("Name, category, and unit cannot be empty.".into());
                self.editing_product = Some((index, product));
                return;
            }
            if !product.quantity.is_finite() || product.quantity < 0.0 || !product.minimum.is_finite() || product.minimum < 0.0 {
                self.notice = Some("Amount and minimum stock must be zero or greater.".into());
                self.editing_product = Some((index, product));
                return;
            }
            if let Some(existing) = self.data.products.get(index) {
                let old_name = existing.name.clone();
                for item in self.data.shopping.iter_mut().filter(|item| item.name.eq_ignore_ascii_case(&old_name)) {
                    item.name = product.name.trim().to_owned();
                    item.unit = product.unit.trim().to_owned();
                }
                self.data.products[index] = Product {
                    name: product.name.trim().to_owned(),
                    category: product.category.trim().to_owned(),
                    unit: product.unit.trim().to_owned(),
                    ..product
                };
                self.persist();
            }
        } else {
            self.editing_product = Some((index, product));
        }
    }

    fn show_pantry(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Find an item");
            ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Search pantry...").desired_width(230.0));
        });
        ui.add_space(12.0);
        egui::Frame::group(ui.style()).fill(egui::Color32::WHITE).corner_radius(10.0).inner_margin(egui::Margin::same(12)).show(ui, |ui| {
            ui.label(egui::RichText::new("Add a pantry item").strong());
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.new_product).hint_text("Item name").desired_width(200.0));
                ui.add(egui::TextEdit::singleline(&mut self.new_quantity).hint_text("Amount").desired_width(72.0));
                if ui.button("Add item").clicked() {
                    let name = self.new_product.trim().to_owned();
                    let parsed = self.new_quantity.trim().parse::<f64>();
                    match parsed {
                        Ok(quantity) if !name.is_empty() && quantity.is_finite() && quantity > 0.0 => {
                            self.data.products.push(Product { name, quantity, unit: "item".into(), minimum: 1.0, category: "General".into() });
                            self.new_product.clear();
                            self.persist();
                        }
                        Ok(_) if name.is_empty() => self.notice = Some("Enter an item name first.".into()),
                        Ok(_) => self.notice = Some("Stock amount must be greater than zero.".into()),
                        Err(_) => self.notice = Some("Enter a valid positive stock amount.".into()),
                    }
                }
            });
        });
        ui.add_space(14.0);
        let mut confirm_removal = false;
        let mut cancel_removal = false;
        if let Some(index) = self.pending_product_removal {
            if let Some(product_name) = self.data.products.get(index).map(|product| product.name.clone()) {
                egui::Frame::group(ui.style())
                    .fill(egui::Color32::from_rgb(255, 248, 239))
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(format!("Remove {product_name} from stock?"));
                            if ui.button("Remove item").clicked() { confirm_removal = true; }
                            if ui.button("Cancel").clicked() { cancel_removal = true; }
                        });
                    });
                ui.add_space(10.0);
            } else {
                self.pending_product_removal = None;
            }
        }
        if confirm_removal {
            if let Some(index) = self.pending_product_removal.take() {
                self.data.products.remove(index);
                self.persist();
            }
        } else if cancel_removal {
            self.pending_product_removal = None;
        }
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Your pantry").size(18.0).strong());
            ui.label(egui::RichText::new(format!("{} items", self.data.products.len())).size(12.0).color(egui::Color32::from_rgb(82, 94, 84)));
        });
        ui.add_space(8.0);
        let filter = self.search.to_lowercase();
        let mut changed = false;
        let mut edit_request = None;
        let mut remove_request = None;
        egui::Grid::new("pantry_grid").num_columns(5).spacing([14.0, 11.0]).striped(true).show(ui, |ui| {
            ui.strong("ITEM"); ui.strong("CATEGORY"); ui.strong("QUANTITY"); ui.strong("MINIMUM"); ui.strong("ACTIONS"); ui.end_row();
            for index in 0..self.data.products.len() {
                if !self.data.products[index].name.to_lowercase().contains(&filter) { continue; }
                let product = &mut self.data.products[index];
                ui.label(egui::RichText::new(&product.name).strong());
                ui.label(&product.category);
                ui.horizontal(|ui| {
                    if ui.small_button("−").clicked() { product.quantity = (product.quantity - 1.0).max(0.0); changed = true; }
                    ui.label(format!("{} {}", format_amount(product.quantity), product.unit));
                    if ui.small_button("+").clicked() { product.quantity += 1.0; changed = true; }
                });
                let is_low = product.quantity <= product.minimum;
                ui.label(egui::RichText::new(format_amount(product.minimum)).color(if is_low { egui::Color32::from_rgb(176, 108, 40) } else { egui::Color32::from_rgb(82, 94, 84) }));
                ui.horizontal(|ui| {
                    if ui.small_button("Edit").clicked() { edit_request = Some((index, product.clone())); }
                    if ui.small_button("Remove").clicked() { remove_request = Some(index); }
                    if ui.small_button("Add to list").clicked() {
                        let item = ShoppingItem { name: product.name.clone(), quantity: 1.0, unit: product.unit.clone(), checked: false };
                        if let Some(existing) = self.data.shopping.iter_mut().find(|i| i.name.eq_ignore_ascii_case(&item.name) && !i.checked) { existing.quantity += 1.0; }
                        else { self.data.shopping.push(item); }
                        changed = true;
                    }
                });
                ui.end_row();
            }
        });
        if let Some((index, product)) = edit_request { self.editing_product = Some((index, product)); }
        if let Some(index) = remove_request { self.pending_product_removal = Some(index); }
        if changed { self.persist(); }
    }

    fn show_shopping(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Plan your next shop").size(18.0).strong());
            ui.label(egui::RichText::new("Check items off as you go").size(12.0).color(egui::Color32::from_rgb(82, 94, 84)));
        });
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.new_shopping).hint_text("What do you need?").desired_width(230.0));
            ui.add(egui::TextEdit::singleline(&mut self.new_shopping_quantity).hint_text("Amount").desired_width(72.0));
            if ui.button("Add to list").clicked() {
                let name = self.new_shopping.trim().to_owned();
                let amount = self.new_shopping_quantity.trim().parse::<f64>();
                match amount {
                    Ok(amount) if amount.is_finite() && amount > 0.0 && !name.is_empty() => {
                        self.add_shopping(name, amount, "item".into());
                        self.new_shopping.clear();
                    }
                    Ok(_) if name.is_empty() => self.notice = Some("Enter an item name first.".into()),
                    _ => self.notice = Some("Shopping amount must be greater than zero.".into()),
                }
            }
            if ui.button("Add missing stock").clicked() { self.add_missing_stock_to_shopping(); }
            if ui.button("Remove checked").clicked() {
                self.data.shopping.retain(|item| !item.checked);
                self.persist();
            }
        });
        ui.add_space(16.0);
        let mut changed = false;
        let mut remove = None;
        for (index, item) in self.data.shopping.iter_mut().enumerate() {
            egui::Frame::group(ui.style()).fill(egui::Color32::WHITE).corner_radius(8.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut item.checked, "").changed() { changed = true; }
                    let text = format!("{}  ·  {} {}", item.name, format_amount(item.quantity), item.unit);
                    let label = if item.checked { egui::RichText::new(text).strikethrough() } else { egui::RichText::new(text) };
                    ui.label(label.color(if item.checked { egui::Color32::from_rgb(82, 94, 84) } else { egui::Color32::from_rgb(46, 57, 48) }));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Remove").clicked() { remove = Some(index); }
                    });
                });
            });
            ui.add_space(4.0);
        }
        if let Some(index) = remove { self.data.shopping.remove(index); changed = true; }
        if self.data.shopping.is_empty() { ui.label("Your list is clear. Add an item whenever you need one."); }
        if changed { self.persist(); }
    }

    fn show_recipes(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Your household recipes").size(18.0).strong());
            if self.editing_recipe.is_none() && ui.button("Add recipe").clicked() {
                self.recipe_name.clear();
                self.recipe_note.clear();
                self.recipe_ingredients.clear();
                self.editing_recipe = Some(usize::MAX);
            }
        });
        ui.label(egui::RichText::new("Add or edit recipes and ingredients, then send missing items to your shopping list.").size(13.0).color(egui::Color32::from_rgb(75, 88, 77)));
        ui.add_space(12.0);

        let mut save_recipe = false;
        let mut cancel_edit = false;
        if self.editing_recipe.is_some() {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::WHITE)
                .inner_margin(egui::Margin::same(14))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(if self.editing_recipe == Some(usize::MAX) { "New recipe" } else { "Edit recipe" }).strong());
                    ui.add(egui::TextEdit::singleline(&mut self.recipe_name).hint_text("Recipe name").desired_width(360.0));
                    ui.add(egui::TextEdit::singleline(&mut self.recipe_note).hint_text("Short description (optional)").desired_width(460.0));
                    ui.label("Ingredients: one per line as name, amount, unit");
                    ui.add(egui::TextEdit::multiline(&mut self.recipe_ingredients).desired_width(460.0).desired_rows(5));
                    ui.horizontal(|ui| {
                        if ui.button("Save recipe").clicked() { save_recipe = true; }
                        if ui.button("Cancel").clicked() { cancel_edit = true; }
                    });
                });
        }
        if cancel_edit { self.editing_recipe = None; }
        if save_recipe {
            match parse_recipe_ingredients(&self.recipe_ingredients) {
                Ok(ingredients) if !self.recipe_name.trim().is_empty() && !ingredients.is_empty() => {
                    let recipe = Recipe {
                        name: self.recipe_name.trim().to_owned(),
                        note: self.recipe_note.trim().to_owned(),
                        ingredients,
                    };
                    if let Some(index) = self.editing_recipe.take() {
                        if index == usize::MAX { self.data.recipes.push(recipe); }
                        else if let Some(existing) = self.data.recipes.get_mut(index) { *existing = recipe; }
                    }
                    self.persist();
                }
                Ok(_) => self.notice = Some("Enter a recipe name and at least one ingredient.".into()),
                Err(error) => self.notice = Some(error),
            }
        }

        let mut edit = None;
        let mut remove = None;
        let recipes = self.data.recipes.clone();
        for (index, recipe) in recipes.iter().enumerate() {
            egui::Frame::group(ui.style()).fill(egui::Color32::WHITE).corner_radius(10.0).inner_margin(egui::Margin::same(14)).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&recipe.name).size(17.0).strong());
                        if !recipe.note.is_empty() {
                            ui.label(egui::RichText::new(&recipe.note).size(12.0).color(egui::Color32::from_rgb(82, 94, 84)));
                        }
                    });
                    if ui.button("Edit").clicked() { edit = Some((index, recipe.clone())); }
                    if ui.button("Remove").clicked() { remove = Some(index); }
                    if ui.button("Add missing to shopping list").clicked() { self.add_missing_ingredients(recipe); }
                });
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    for (name, amount, unit) in &recipe.ingredients {
                        let have = self.data.products.iter().find(|product| product.name.eq_ignore_ascii_case(name)).map_or(0.0, |product| product.quantity);
                        let enough = have >= *amount;
                        ui.label(egui::RichText::new(format!("{} {} {}", format_amount(*amount), unit, name)).size(12.0).color(if enough { egui::Color32::from_rgb(76, 118, 83) } else { egui::Color32::from_rgb(168, 112, 50) }));
                    }
                });
            });
            ui.add_space(8.0);
        }
        if let Some((index, recipe)) = edit {
            self.recipe_name = recipe.name;
            self.recipe_note = recipe.note;
            self.recipe_ingredients = recipe.ingredients.iter()
                .map(|(name, amount, unit)| format!("{name}, {}, {unit}", format_amount(*amount)))
                .collect::<Vec<_>>().join("\n");
            self.editing_recipe = Some(index);
        }
        if let Some(index) = remove {
            self.data.recipes.remove(index);
            if self.editing_recipe == Some(index) { self.editing_recipe = None; }
            self.persist();
        }
    }

    fn show_meal_plan(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Plan simple meals across the week").size(13.0).color(egui::Color32::from_rgb(111, 121, 112)));
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            egui::ComboBox::from_id_salt("meal_plan_day")
                .selected_text(&self.selected_day)
                .show_ui(ui, |ui| {
                    for day in ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"] {
                        ui.selectable_value(&mut self.selected_day, day.to_owned(), day);
                    }
                });
            ui.add(egui::TextEdit::singleline(&mut self.new_meal).hint_text("Meal or recipe").desired_width(240.0));
            if ui.button("Add to plan").clicked() {
                let meal = self.new_meal.trim().to_owned();
                if meal.is_empty() {
                    self.notice = Some("Enter a meal or recipe name first.".into());
                } else {
                    self.data.meal_plan.push(MealPlanEntry { day: self.selected_day.clone(), meal });
                    self.new_meal.clear();
                    self.persist();
                }
            }
        });
        ui.add_space(16.0);
        let mut remove = None;
        for day in ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"] {
            ui.label(egui::RichText::new(day).size(16.0).strong());
            let meals: Vec<(usize, String)> = self.data.meal_plan.iter().enumerate()
                .filter(|(_, entry)| entry.day == day)
                .map(|(index, entry)| (index, entry.meal.clone()))
                .collect();
            if meals.is_empty() {
                ui.label(egui::RichText::new("No meal planned").size(12.0).color(egui::Color32::from_rgb(82, 94, 84)));
            }
            for (index, meal) in meals {
                ui.horizontal(|ui| {
                    ui.label(format!("•  {meal}"));
                    if ui.small_button("Remove").clicked() { remove = Some(index); }
                });
            }
            ui.add_space(8.0);
        }
        if let Some(index) = remove {
            self.data.meal_plan.remove(index);
            self.persist();
        }
    }

    fn show_chores(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.label(egui::RichText::new("Assign chores to household members and set reminder dates.").size(13.0).color(egui::Color32::from_rgb(75, 88, 77)));
        ui.add_space(12.0);

        egui::Frame::group(ui.style()).fill(egui::Color32::WHITE).inner_margin(egui::Margin::same(12)).show(ui, |ui| {
            ui.label(egui::RichText::new("Household members").size(16.0).strong());
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.new_member_name).hint_text("Name").desired_width(180.0));
                ui.add(egui::TextEdit::singleline(&mut self.new_member_email).hint_text("Email address (optional)").desired_width(240.0));
                if ui.button("Add person").clicked() {
                    let name = self.new_member_name.trim().to_owned();
                    let email = self.new_member_email.trim().to_owned();
                    if name.is_empty() {
                        self.notice = Some("Enter a name for this household member.".into());
                    } else if !email.is_empty() && !valid_email(&email) {
                        self.notice = Some("Enter a valid email address, or leave it blank.".into());
                    } else if self.data.members.iter().any(|member| member.name.eq_ignore_ascii_case(&name)) {
                        self.notice = Some("That household member already exists.".into());
                    } else {
                        self.data.members.push(HouseholdMember { name, email });
                        self.new_member_name.clear();
                        self.new_member_email.clear();
                        self.persist();
                    }
                }
            });
            let mut remove_member = None;
            for (index, member) in self.data.members.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&member.name).strong());
                    if !member.email.is_empty() {
                        ui.label(egui::RichText::new(&member.email).size(12.0).color(egui::Color32::from_rgb(82, 94, 84)));
                    }
                    if ui.small_button("Remove person").clicked() { remove_member = Some(index); }
                });
            }
            if let Some(index) = remove_member {
                let removed_name = self.data.members.remove(index).name;
                for chore in &mut self.data.chores {
                    if chore.assigned_to.as_deref() == Some(removed_name.as_str()) { chore.assigned_to = None; }
                }
                self.persist();
            }
        });

        ui.add_space(12.0);
        ui.label(egui::RichText::new("Add a chore").size(16.0).strong());
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.new_chore).hint_text("Chore name").desired_width(210.0));
            egui::ComboBox::from_id_salt("chore_frequency")
                .selected_text(&self.selected_frequency)
                .show_ui(ui, |ui| {
                    for frequency in ["Hourly", "Daily", "Weekly", "Monthly", "Yearly", "Adaptive", "Manually"] {
                        ui.selectable_value(&mut self.selected_frequency, frequency.to_owned(), frequency);
                    }
                });
            egui::ComboBox::from_id_salt("chore_assignee")
                .selected_text(if self.selected_assignee.is_empty() { "No assignee" } else { &self.selected_assignee })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_assignee, String::new(), "No assignee");
                    for member in &self.data.members {
                        ui.selectable_value(&mut self.selected_assignee, member.name.clone(), &member.name);
                    }
                });
            ui.add(egui::TextEdit::singleline(&mut self.new_chore_reminder_date).hint_text("Reminder date YYYY-MM-DD").desired_width(190.0));
            if ui.button("Add chore").clicked() {
                let name = self.new_chore.trim().to_owned();
                let date_text = self.new_chore_reminder_date.trim();
                let reminder_date = if date_text.is_empty() { None } else if parse_reminder_date(date_text).is_some() {
                    Some(date_text.to_owned())
                } else {
                    self.notice = Some("Use a valid reminder date in YYYY-MM-DD format.".into());
                    return;
                };
                if name.is_empty() {
                    self.notice = Some("Enter a chore name.".into());
                } else {
                    self.data.chores.push(Chore {
                        name,
                        frequency: self.selected_frequency.clone(),
                        completed: false,
                        assigned_to: (!self.selected_assignee.is_empty()).then(|| self.selected_assignee.clone()),
                        reminder_date,
                    });
                    self.new_chore.clear();
                    self.new_chore_reminder_date.clear();
                    self.persist();
                }
            }
        });

        let today = chrono::Utc::now().date_naive();
        let upcoming: Vec<&Chore> = self.data.chores.iter().filter(|chore| {
            !chore.completed && chore.reminder_date.as_deref().and_then(parse_reminder_date)
                .is_some_and(|date| date <= today + chrono::Duration::days(7))
        }).collect();
        if !upcoming.is_empty() {
            ui.add_space(16.0);
            ui.label(egui::RichText::new("Reminders").size(16.0).strong());
            for chore in upcoming {
                let date = chore.reminder_date.as_deref().unwrap_or_default();
                let status = reminder_status(date, today).unwrap_or("Upcoming");
                ui.label(egui::RichText::new(format!("{status}: {} — {date}", chore.name)).color(egui::Color32::from_rgb(126, 76, 25)));
            }
        }

        ui.add_space(12.0);
        let members = self.data.members.clone();
        let mut changed = false;
        let mut remove = None;
        let mut email_request = None;
        for (index, chore) in self.data.chores.iter_mut().enumerate() {
            egui::Frame::group(ui.style()).fill(egui::Color32::WHITE).corner_radius(8.0).inner_margin(egui::Margin::same(10)).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.checkbox(&mut chore.completed, "").changed() { changed = true; }
                    let label = if chore.completed { egui::RichText::new(&chore.name).strikethrough() } else { egui::RichText::new(&chore.name) };
                    ui.label(label.color(if chore.completed { egui::Color32::from_rgb(82, 94, 84) } else { egui::Color32::from_rgb(46, 57, 48) }));
                    ui.label(egui::RichText::new(&chore.frequency).size(12.0).color(egui::Color32::from_rgb(82, 94, 84)));
                    if let Some(name) = chore.assigned_to.as_deref() {
                        ui.label(egui::RichText::new(format!("For: {name}")).size(12.0).color(egui::Color32::from_rgb(55, 96, 66)));
                    }
                    if let Some(date) = chore.reminder_date.as_deref() {
                        let status = reminder_status(date, today).unwrap_or("Scheduled");
                        ui.label(egui::RichText::new(format!("{status} · {date}")).size(12.0).color(egui::Color32::from_rgb(126, 76, 25)));
                    }
                    if let Some(member) = chore.assigned_to.as_deref().and_then(|name| members.iter().find(|member| member.name == name)) {
                        if !member.email.is_empty() && ui.small_button("Draft email reminder").clicked() {
                            email_request = Some(email_draft_uri(&member.email, &chore.name, chore.reminder_date.as_deref()));
                        }
                    }
                    if ui.small_button("Remove").clicked() { remove = Some(index); }
                });
            });
            ui.add_space(4.0);
        }
        if let Some(uri) = email_request {
            ctx.open_url(egui::OpenUrl::new_tab(uri));
            self.notice = Some("Email draft opened in your mail app; it is not sent automatically.".into());
        }
        if let Some(index) = remove { self.data.chores.remove(index); changed = true; }
        if self.data.chores.is_empty() { ui.label("No chores yet. Add a task to get started."); }
        if changed { self.persist(); }
    }

    fn write_screenshot(path: &std::path::Path, image: &egui::ColorImage) -> Result<(), String> {
        let width = image.size[0] as u32;
        let height = image.size[1] as u32;
        let pixels: Vec<u8> = image.pixels.iter().flat_map(|pixel| [pixel.r(), pixel.g(), pixel.b(), pixel.a()]).collect();
        let buffer = image::RgbaImage::from_raw(width, height, pixels).ok_or_else(|| "could not encode screenshot pixels".to_owned())?;
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).map_err(|error| error.to_string())?; }
        buffer.save(path).map_err(|error| error.to_string())
    }
}

impl eframe::App for PantryDesk {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());
        if let Some(path) = self.screenshot_path.as_ref() {
            let screenshot = ctx.input(|input| input.raw.events.iter().find_map(|event| match event {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            }));
            if let Some(image) = screenshot {
                match Self::write_screenshot(path, &image) {
                    Ok(()) => std::process::exit(0),
                    Err(error) => {
                        eprintln!("Could not save screenshot: {error}");
                        std::process::exit(1);
                    }
                }
            }
            self.frames += 1;
            if self.frames >= 120 && !self.screenshot_requested {
                self.screenshot_requested = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            }
            ctx.request_repaint_after(Duration::from_millis(8));
        }

        self.show_sidebar(ctx);
        egui::CentralPanel::default().frame(egui::Frame::central_panel(&ctx.style()).fill(egui::Color32::from_rgb(247, 248, 244)).inner_margin(egui::Margin::same(24))).show(ctx, |ui| {
            self.show_header(ui);
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                match self.section {
                    Section::Overview => self.show_overview(ui),
                    Section::Pantry => self.show_pantry(ui),
                    Section::Shopping => self.show_shopping(ui),
                    Section::Recipes => self.show_recipes(ui),
                    Section::MealPlan => self.show_meal_plan(ui),
                    Section::Chores => self.show_chores(ui, ctx),
                }
                if let Some(message) = &self.notice {
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new(message).size(12.0).color(egui::Color32::from_rgb(72, 111, 82)));
                }
            });
        });
        self.show_product_editor(ctx);
    }
}

fn parse_recipe_ingredients(input: &str) -> Result<Vec<(String, f64, String)>, String> {
    let mut ingredients = Vec::new();
    for (line_number, line) in input.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        if fields.len() != 3 || fields[0].is_empty() || fields[2].is_empty() {
            return Err(format!("Ingredient line {} must be: name, amount, unit.", line_number + 1));
        }
        let amount = fields[1].parse::<f64>().ok().filter(|amount| amount.is_finite() && *amount > 0.0)
            .ok_or_else(|| format!("Ingredient line {} needs an amount greater than zero.", line_number + 1))?;
        ingredients.push((fields[0].to_owned(), amount, fields[2].to_owned()));
    }
    Ok(ingredients)
}

fn valid_email(email: &str) -> bool {
    let Some((name, domain)) = email.split_once('@') else { return false; };
    !name.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
        && !email.chars().any(char::is_whitespace)
}

fn parse_reminder_date(value: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn reminder_status(value: &str, today: chrono::NaiveDate) -> Option<&'static str> {
    let date = parse_reminder_date(value)?;
    if date < today {
        Some("Overdue")
    } else if date == today {
        Some("Due today")
    } else if date <= today + chrono::Duration::days(7) {
        Some("Due soon")
    } else {
        None
    }
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn email_draft_uri(email: &str, chore: &str, reminder_date: Option<&str>) -> String {
    let date_line = reminder_date.map_or(String::new(), |date| format!("\nReminder date: {date}"));
    let body = format!("This is a reminder for the household chore: {chore}.{date_line}");
    format!("mailto:{email}?subject={}&body={}", percent_encode("Household chore reminder"), percent_encode(&body))
}

fn missing_stock_amount(stock: f64, minimum: f64) -> f64 {
    ((minimum - stock).max(0.0) * 100.0).round() / 100.0
}

fn format_amount(amount: f64) -> String {
    if amount.fract() == 0.0 { format!("{amount:.0}") } else { amount.to_string() }
}

#[cfg(test)]
mod tests {
    use super::{email_draft_uri, format_amount, missing_stock_amount, parse_recipe_ingredients, reminder_status, valid_email};

    #[test]
    fn stock_shortfall_is_fractional_and_never_negative() {
        assert_eq!(missing_stock_amount(1.25, 3.0), 1.75);
        assert_eq!(missing_stock_amount(1.234, 2.0), 0.77);
        assert_eq!(missing_stock_amount(4.0, 3.0), 0.0);
        assert_eq!(missing_stock_amount(3.0, 3.0), 0.0);
    }

    #[test]
    fn recipe_ingredients_parse_and_reject_invalid_amounts() {
        let parsed = parse_recipe_ingredients("Oats, 1.5, cup\nApples, 2, pieces").unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], ("Oats".into(), 1.5, "cup".into()));
        assert!(parse_recipe_ingredients("Oats, 0, cup").is_err());
        assert!(parse_recipe_ingredients("Oats, one cup").is_err());
    }

    #[test]
    fn chore_reminders_are_classified_by_date() {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 11, 12).unwrap();
        assert_eq!(reminder_status("2026-11-11", today), Some("Overdue"));
        assert_eq!(reminder_status("2026-11-12", today), Some("Due today"));
        assert_eq!(reminder_status("2026-11-19", today), Some("Due soon"));
        assert_eq!(reminder_status("2026-11-20", today), None);
        assert_eq!(reminder_status("not-a-date", today), None);
    }

    #[test]
    fn email_reminder_draft_is_encoded_and_requires_a_valid_email() {
        assert!(valid_email("household@example.com"));
        assert!(!valid_email("not-an-email"));
        let uri = email_draft_uri("household@example.com", "Take out bins", Some("2026-11-12"));
        assert!(uri.starts_with("mailto:household@example.com?subject="));
        assert!(uri.contains("Take%20out%20bins"));
        assert!(uri.contains("2026-11-12"));
    }

    #[test]
    fn amounts_are_displayed_without_unneeded_decimal_zeros() {
        assert_eq!(format_amount(2.0), "2");
        assert_eq!(format_amount(2.5), "2.5");
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn start() -> Result<(), wasm_bindgen::JsValue> {
    let window = web_sys::window().ok_or_else(|| wasm_bindgen::JsValue::from_str("browser window is unavailable"))?;
    let document = window.document().ok_or_else(|| wasm_bindgen::JsValue::from_str("browser document is unavailable"))?;
    let canvas = document.get_element_by_id("the_canvas_id")
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("app canvas was not found"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    let runner = Box::leak(Box::new(eframe::WebRunner::new()));
    runner.start(
        canvas,
        eframe::WebOptions::default(),
        Box::new(|_creation_context| Ok(Box::new(PantryDesk::new(PathBuf::new(), None)))),
    ).await
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn data_file() -> PathBuf {
    if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(path).join("pantry-desk").join("household.json");
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    home.join(".local").join("share").join("pantry-desk").join("household.json")
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let mut args = std::env::args().skip(1);
    let mut screenshot_path = None;
    while let Some(argument) = args.next() {
        if argument == "--screenshot" {
            let Some(path) = args.next() else {
                eprintln!("--screenshot requires a PNG file path");
                std::process::exit(2);
            };
            screenshot_path = Some(PathBuf::from(path));
        }
    }
    let screenshot_mode = screenshot_path.is_some();
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1060.0, 720.0])
        .with_min_inner_size([720.0, 500.0])
        .with_title("Home Stock");
    if screenshot_mode { viewport = viewport.with_active(false); }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native(
        "Home Stock",
        options,
        Box::new(move |_creation_context| Ok(Box::new(PantryDesk::new(data_file(), screenshot_path)))),
    )
}
