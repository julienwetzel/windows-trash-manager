use crate::NOTICE;
use chrono::offset::Local;
use chrono::{Duration, NaiveDateTime, TimeZone};
use trash::os_limited::{list, purge_all};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AppConfig {
    time_threshold: i64,
}

impl AppConfig {}

impl Default for AppConfig {
    fn default() -> Self {
        Self { time_threshold: 15 }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ConsoleApp {
    console_txt: String,
    app_config: AppConfig,
}

impl ConsoleApp {
    pub fn add_text(&mut self, text: &str) {
        self.console_txt.push_str(text);
    }
}

impl Default for ConsoleApp {
    fn default() -> Self {
        Self {
            console_txt: NOTICE.to_owned(),
            app_config: AppConfig { time_threshold: 30 },
        }
    }
}

// We derive Deserialize/Serialize so we can persist app state on shutdown.
// if we add new fields, give them default values when deserializing old state
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    console_app: ConsoleApp,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            console_app: ConsoleApp::default(),
        }
    }
}

//################################# UI AREA ###################################

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}
//############################# UI PANEL AREA #################################

impl eframe::App for TemplateApp {
    // Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //let Self { console_txt } = self;
        //___________________________ TOPBOTTOMPANEL __________________________
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
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
                            egui::DragValue::new(&mut self.console_app.app_config.time_threshold)
                                .speed(0.1)
                                .suffix(" jours")
                                .clamp_range(1.0..=365.0),
                        );
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
                            analyser(&mut self.console_app);
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
                            supprimer_definitivement(&mut self.console_app);
                        }
                    },
                );

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    egui::widgets::global_dark_light_mode_buttons(ui);
                    ui.separator();
                });
                ui.separator();
                ui.hyperlink("https://github.com/julienwetzel/windows-trash-manager");
                ui.add(egui::github_link_file!(
                    "https://github.com/julienwetzel/windows-trash-manager/blob/main/src/app.rs",
                    "Code source"
                ));
            });

        //____________________________CENTRALPANEL_____________________________
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom(true) // Ajoutez cette ligne pour coller le scroll en bas
                .show(ui, |ui| {
                    let mut text: &str = &self.console_app.console_txt;
                    ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut text));
                });
        });
    }
}

//########################### BUTTONS FUNCTIONS AREA ##########################

//___________________FUNCTION BUTTON SUPPRIMER_DEFINITIVEMENT__________________
fn supprimer_definitivement(console_app: &mut ConsoleApp) {
    let now = Local::now().naive_local();
    let threshold = Duration::days(console_app.app_config.time_threshold);
    let trash_items = match list() {
        Ok(items) => items,
        Err(e) => {
            console_app.add_text(&format!(
                "**** Erreur lors de la récupération des éléments de la corbeille: {} ****\n",
                e
            ));
            return;
        }
    };

    console_app.add_text("SUPPRESSION DÉFINITIVE\n\n");

    for item in &trash_items {
        let time_deleted = match NaiveDateTime::from_timestamp_opt(item.time_deleted, 0) {
            Some(time) => time,
            None => {
                console_app.add_text("**** Erreur lors de la conversion de l'horodatage ****\n");
                continue;
            }
        };
        let time_deleted_local = Local.from_utc_datetime(&time_deleted);

        if now.signed_duration_since(time_deleted_local.naive_local()) > threshold {
            let formatted_time_deleted = time_deleted_local.format("%d.%m.%Y %H:%M").to_string();
            let message = format!(
                "Nom : {} \n\
                Supprimé le : {}\n\n",
                item.name, formatted_time_deleted
            );
            console_app.add_text(&message);
        }
    }

    if let Err(e) = purge_all(trash_items.into_iter().filter(|item| {
        let time_deleted = match NaiveDateTime::from_timestamp_opt(item.time_deleted, 0) {
            Some(time) => time,
            None => {
                console_app.add_text("**** Erreur lors de la conversion de l'horodatage ****\n");
                return false;
            }
        };
        let time_deleted_local = Local.from_utc_datetime(&time_deleted);
        now.signed_duration_since(time_deleted_local.naive_local()) > threshold
    })) {
        console_app.add_text(&format!("**** Erreur lors de la purge: {} ****\n", e));
    }
}

//__________________________FUNCTION BUTTON ANALYSER___________________________
fn analyser(console_app: &mut ConsoleApp) {
    let now = Local::now().naive_local();
    let threshold = Duration::days(console_app.app_config.time_threshold);
    let trash_items = match list() {
        Ok(items) => items,
        Err(e) => {
            console_app.add_text(&format!(
                "**** Erreur lors de la récupération des éléments de la corbeille: {} ****\n",
                e
            ));
            return;
        }
    };

    let title = format!(
        "LISTE DES ELEMENTS SUPPRIMÉS IL Y A PLUS DE {} JOURS\n\n",
        console_app.app_config.time_threshold
    );
    console_app.add_text(&title);

    for item in &trash_items {
        let time_deleted = match NaiveDateTime::from_timestamp_opt(item.time_deleted, 0) {
            Some(time) => time,
            None => {
                console_app.add_text("**** Erreur lors de la conversion de l'horodatage ****\n");
                continue;
            }
        };
        let time_deleted_local = Local.from_utc_datetime(&time_deleted);

        if now.signed_duration_since(time_deleted_local.naive_local()) > threshold {
            let formatted_time_deleted = time_deleted_local.format("%d.%m.%Y %H:%M").to_string();
            let message = format!(
                "Nom : {} \n\
                Supprimé le : {}\n\n",
                item.name, formatted_time_deleted
            );
            console_app.add_text(&message);
        }
    }
}
