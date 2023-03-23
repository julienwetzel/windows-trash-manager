use chrono::{offset::Local, Duration, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};
use trash::os_limited::{list, purge_all};

use crate::{CircularBuffer, GitHubInfo, NOTICE};

impl<T> Serialize for CircularBuffer<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data = self.iter().collect::<Vec<&T>>();
        let capacity = self.buffer.capacity();
        (capacity, data).serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for CircularBuffer<T>
where
    T: Deserialize<'de> + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (capacity, data): (usize, Vec<T>) = Deserialize::deserialize(deserializer)?;
        let mut buffer = CircularBuffer::new(capacity);

        for item in data {
            buffer.push(item);
        }

        Ok(buffer)
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ConfigApp {
    time_threshold: u8,
    max_console_lines: u16,
}

impl Default for ConfigApp {
    fn default() -> Self {
        Self {
            time_threshold: 30,
            max_console_lines: 1000,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ConsoleApp {
    console_queue: CircularBuffer<String>,
}

impl ConsoleApp {
    pub fn get_last_console_messages(&self, count: usize) -> Vec<&String> {
        if self.console_queue.is_empty() {
            return Vec::new();
        }

        let messages = self.console_queue.iter().collect::<Vec<&String>>();
        messages.into_iter().take(count).collect()
    }

    pub fn add_to_buffer(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        for line in text.lines() {
            self.console_queue.push(line.to_string());
        }
    }

    pub fn flush_storage(&mut self) {
        if self.console_queue.is_empty() {
            return;
        }

        self.console_queue.clear();
    }
}

impl Default for ConsoleApp {
    fn default() -> Self {
        let config_app = ConfigApp::default();
        Self {
            console_queue: CircularBuffer::new(config_app.max_console_lines.into()),
        }
    }
}

/*
We derive Deserialize/Serialize so we can persist app state on shutdown.
if we add new fields, give them default values when deserializing old state
*/
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct TemplateApp {
    config_app: ConfigApp,
    console_app: ConsoleApp,
}

//################################# UI AREA ###################################

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        let mut template_app: TemplateApp = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        template_app.console_app.add_to_buffer(NOTICE);
        template_app
    }
}

//############################# UI PANEL AREA #################################

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            config_app,
            console_app,
        } = self;
        //___________________________ TOPBOTTOMPANEL __________________________
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });
        //______________________________ SIDEPANEL ____________________________
        egui::SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                egui::warn_if_debug_build(ui);
                ui.add_space(8.0);
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        ui.add(egui::Label::new("Préserver"));
                    },
                );

                // *** DAYS USER INPUT ***
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        ui.add(
                            egui::DragValue::new(&mut config_app.time_threshold)
                                .speed(0.1)
                                .suffix(" jours")
                                .clamp_range(1.0..=255.0),
                        );
                    },
                );

                // *** CLEAR CONSOLE QUEUE BUTTON ***
                ui.add_space(8.0);
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        if ui.button("Vider la console").clicked() {
                            console_app.flush_storage();
                        }
                    },
                );
                // *** CLEAR MEMORY BUTTON ***
                ui.add_space(8.0);
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        if ui.button("Vider le frame").clicked() {
                            if let Some(storage) = frame.storage_mut() {
                                // Effacer la persistance enregistrée
                                clear_cache(storage);
                                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
                            }
                        }
                    },
                );

                ui.add_space(8.0);
                ui.separator();

                // *** BUTTON ANALYSER ***
                ui.add_space(8.0);
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        if ui.button("Analyser").clicked() {
                            analyser(console_app, config_app);
                        }
                    },
                );

                // *** BUTTON SUPPRIMER_DEFINITIVEMENT ***
                ui.add_space(8.0);
                let btn_label = "Supprimer définitivement";
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        let button_response = ui.button(btn_label);
                        if button_response.clicked() {
                            supprimer_definitivement(console_app, config_app);
                        }
                    },
                );

                ui.horizontal(|ui| {
                    ui.label("Write something: ");
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    egui::widgets::global_dark_light_mode_buttons(ui);
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add(egui::Hyperlink::from_label_and_url(
                            "GitHub",
                            GitHubInfo::default().url,
                        ));
                        ui.label(egui::special_emojis::GITHUB.to_string());
                        ui.add(egui::github_link_file!(
                            GitHubInfo::default().url_blob,
                            "Code source"
                        ));
                    });
                });
            });

        //____________________________CENTRALPANEL_____________________________
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let lines = self
                        .console_app
                        .get_last_console_messages(config_app.max_console_lines.into());
                    let mut text = lines
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>()
                        .join("\n");
                    ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut text));
                });
        });
    }

    // Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

//########################### BUTTONS FUNCTIONS AREA ##########################

pub fn clear_cache(storage: &mut dyn eframe::Storage) {
    let none_ref: &Option<()> = &None;
    eframe::set_value(storage, eframe::APP_KEY, none_ref);
}

//___________________FUNCTION BUTTON SUPPRIMER_DEFINITIVEMENT__________________
fn supprimer_definitivement(console_app: &mut ConsoleApp, config_app: &mut ConfigApp) {
    let elements_to_process = get_elements_to_process(console_app, config_app);

    console_app.add_to_buffer("SUPPRESSION DÉFINITIVE\n\n");

    // Affichage des en-têtes de colonnes
    console_app.add_to_buffer("Nom                                      | Supprimé le          \n");
    console_app.add_to_buffer("-----------------------------------------|---------------------\n");

    for item in &elements_to_process {
        match NaiveDateTime::from_timestamp_opt(item.time_deleted, 0) {
            Some(naive_datetime) => {
                let time_deleted_local = Local.from_utc_datetime(&naive_datetime);
                let formatted_time_deleted =
                    time_deleted_local.format("%d.%m.%Y %H:%M").to_string();
                let message = format!("{: <40}| {: <19}\n", item.name, formatted_time_deleted);
                console_app.add_to_buffer(&message);
            }
            None => {
                console_app
                    .add_to_buffer("**** Erreur lors de la conversion de l'horodatage ****\n");
                return;
            }
        };
    }

    if let Err(e) = purge_all(elements_to_process.into_iter()) {
        console_app.add_to_buffer(&format!("**** Erreur lors de la purge: {} ****\n", e));
    }
}

//__________________________FUNCTION BUTTON ANALYSER___________________________
fn analyser(console_app: &mut ConsoleApp, config_app: &mut ConfigApp) {
    let elements_to_process = get_elements_to_process(console_app, config_app);

    console_app.add_to_buffer("ANALYSE\n\n");

    for item in &elements_to_process {
        match NaiveDateTime::from_timestamp_opt(item.time_deleted, 0) {
            Some(naive_datetime) => {
                let time_deleted_local = Local.from_utc_datetime(&naive_datetime);
                let formatted_time_deleted =
                    time_deleted_local.format("%d.%m.%Y %H:%M").to_string();
                let message = format!(
                    "Nom : {} \n\
                    Supprimé le : {}\n\n",
                    item.name, formatted_time_deleted
                );
                console_app.add_to_buffer(&message);
            }
            None => {
                console_app.add_to_buffer("Erreur lors de la conversion de la date.");
                return;
            }
        };
    }

    let total_items = elements_to_process.len();
    console_app.add_to_buffer(&format!("Total d'éléments à traiter : {}\n", total_items));
}

//______________________FUNCTION GET_ELEMENTS_TO_PROCESS_______________________
fn get_elements_to_process(
    console_app: &mut ConsoleApp,
    config_app: &mut ConfigApp,
) -> Vec<trash::TrashItem> {
    let now = Local::now().naive_local();
    let duration = config_app.time_threshold as i64;
    let threshold = Duration::days(duration);
    let trash_items = match list() {
        Ok(items) => items,
        Err(e) => {
            console_app.add_to_buffer(&format!(
                "**** Erreur lors de la récupération des éléments de la corbeille: {} ****\n",
                e
            ));
            return vec![];
        }
    };

    trash_items
        .into_iter()
        .filter(|item| {
            let time_deleted = match NaiveDateTime::from_timestamp_opt(item.time_deleted, 0) {
                Some(time) => time,
                None => {
                    console_app
                        .add_to_buffer("**** Erreur lors de la conversion de l'horodatage ****\n");
                    return false;
                }
            };
            let time_deleted_local = Local.from_utc_datetime(&time_deleted);
            now.signed_duration_since(time_deleted_local.naive_local()) > threshold
        })
        .collect()
}
