use log::{error, info};

use std::fmt;
use std::path::Path;

use crate::encrypt::encrypt_file_with_inmemory_key;
use crate::folder::*;

use crate::keys_management::*;
use egui::Color32;
use egui::{Context, RichText, Ui, Vec2};
use egui_extras::{Size, StripBuilder};

use flowync::error::Cause;
use flowync::Flower;

use isahc::prelude::*;

type TypedFlower = Flower<String, String>;

#[derive(Debug, Clone)]
struct AppError {
    _msg: String,
}

impl AppError {
    pub fn new<T>(msg: T) -> AppError
    where
        T: Into<String>,
    {
        AppError { _msg: msg.into() }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Application Error : {}", &self._msg)
    }
}
impl std::error::Error for AppError {}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct EncrypterApp {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,

    #[serde(skip)]
    selected: Option<Key>,

    #[serde(skip)]
    files_folder: FolderNode,

    #[serde(skip)]
    db: Database,

    #[serde(skip)]
    last_message: String,
    #[serde(skip)]
    is_error: bool,

    is_add_opened: bool,
    key_name: String,
    key_sha1_input: String,

    #[serde(skip)]
    flower: TypedFlower,
}

impl Default for EncrypterApp {
    fn default() -> Self {
        let mut r = FolderNode {
            expanded: false,
            is_folder: true,
            path: ".".into(),
            subfolders: vec![],
            selected: false,
        };

        let db = Database::open_database().expect("cannot open database");

        // expand the first level
        if let Err(e) = expand(&mut r) {
            error!("error in expanding the tree : {}", e);
        }

        Self {
            // Example stuff:
            label: "Encrypter".to_owned(),
            value: 2.7,
            selected: None,
            files_folder: r,
            db,
            last_message: "".to_owned(),
            is_error: false,
            is_add_opened: false,
            key_name: "".to_owned(),
            key_sha1_input: "".to_owned(),
            flower: TypedFlower::new(1),
        }
    }
}

impl EncrypterApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        let ctx = &mut cc.egui_ctx.to_owned();

        EncrypterApp::install_style(ctx);

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    /**
     * install style
     */
    fn install_style(ctx: &mut Context) {
        use egui::FontFamily::Proportional;
        use egui::FontId;
        use egui::TextStyle::*;

        use eframe::egui::Visuals;

        // Get current context style
        let mut style = (*ctx.style()).clone();

        // Redefine text_styles
        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Name("Heading2".into()), FontId::new(26.0, Proportional)),
            (Name("Context".into()), FontId::new(23.0, Proportional)),
            (Body, FontId::new(20.0, Proportional)),
            (Monospace, FontId::new(19.5, Proportional)),
            (Button, FontId::new(23.0, Proportional)),
            (Small, FontId::new(17.0, Proportional)),
        ]
        .into();

        let mut spacing = style.spacing.clone();
        spacing.item_spacing = Vec2 { x: 12.0, y: 7.0 };
        spacing.window_margin = egui::style::Margin {
            left: 8.0,
            right: 8.0,
            top: 8.0,
            bottom: 8.0,
        };
        spacing.button_padding = Vec2 { x: 4.0, y: 1.0 };
        spacing.menu_margin = egui::style::Margin {
            left: 8.0,
            right: 8.0,
            top: 8.0,
            bottom: 8.0,
        };
        spacing.indent = 32.0;
        spacing.interact_size = Vec2 { x: 40.0, y: 18.0 };
        spacing.slider_width = 100.0;
        spacing.text_edit_width = 292.0;
        spacing.icon_width = 23.0;

        spacing.icon_width_inner = 18.0;
        spacing.icon_spacing = 4.0;
        spacing.tooltip_width = 600.0;
        spacing.indent_ends_with_horizontal_line = false;
        spacing.combo_height = 200.0;
        spacing.scroll_bar_width = 14.0;
        spacing.scroll_bar_inner_margin = 6.0;
        spacing.scroll_bar_outer_margin = 1.0;

        style.spacing = spacing;

        style.visuals = Visuals::light();
        style.visuals.collapsing_header_frame = true;

        // Mutate global style with above changes
        ctx.set_style(style);

        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = egui::FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        fonts.font_data.insert(
            "my_font".to_owned(),
            egui::FontData::from_static(include_bytes!("../fonts/static/Rubik-Light.ttf")),
        );

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "my_font".to_owned());

        // Put my font as last fallback for monospace:
        // fonts
        //     .families
        //     .entry(egui::FontFamily::Monospace)
        //     .or_default()
        //     .push("my_font".to_owned());

        // Tell egui to use these fonts:
        ctx.set_fonts(fonts);
    }

    /// recursive function to display files
    fn display_tree(files_folder: &mut FolderNode, ui: &mut Ui) -> crate::Result<()> {
        for ele in &mut files_folder.subfolders {
            let element_name = String::from(ele.name());
            if ele.is_folder {
                let r = ui.collapsing(element_name, |ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    if let Err(e) = EncrypterApp::display_tree(ele, ui) {
                        error!("error in displaying sub tree {}", e);
                    }
                });
                if r.fully_open() && !ele.expanded {
                    expand(ele)?;
                }
            } else {
                ui.checkbox(&mut ele.selected, element_name);
            }
        }
        Ok(())
    }

    fn construct_list(file_folder: &FolderNode, ui: &mut Ui) {
        if file_folder.selected {
            ui.label(RichText::new(file_folder.clone().name()).color(Color32::DARK_BLUE));
        }
        for elem in &file_folder.subfolders {
            EncrypterApp::construct_list(elem, ui);
        }
    }

    fn crypt_selected(
        file_folder: &FolderNode,
        keyname: &String,
        sha1: &String,
        key: &[u8],
    ) -> crate::Result<()> {
        if file_folder.selected {
            let filename = file_folder.name().to_string();

            let folder_name = keyname.clone() + "-" + sha1;
            if !Path::new(&folder_name).try_exists()? {
                std::fs::create_dir(folder_name.clone())?;
            }

            let o_conversion = Path::new(&folder_name)
                .join(filename.clone() + "x")
                .into_os_string()
                .into_string();

            if let Ok(s) = o_conversion {
                let output_filename: String = s;
                encrypt_file_with_inmemory_key(&file_folder.path.clone(), &output_filename, key)?;
                info!(" file {}, encrypted", &output_filename);
            } else {
                let msg = format!("fail to create file for {}", &filename);
                return Err(AppError::new(msg).into());
            }
        }

        for elem in &file_folder.subfolders {
            EncrypterApp::crypt_selected(elem, keyname, sha1, key)?;
        }
        Ok(())
    }

    fn download_key(flower: &TypedFlower, sha1: String) {
        std::thread::spawn({
            let handle = flower.handle();
            // Activate
            handle.activate();
            move || {
                handle.send("start".into());

                let response_result =
                    isahc::get("http://or1.frett27.net/k/".to_string() + &sha1 + "/public.key.pem");

                if let Ok(mut response) = response_result {
                    if response.status().is_success() {
                        let text = response.text(); // read response

                        if let Ok(returned_elements) = text {
                            // Set result and then extract later.
                            handle.set_result(Ok(returned_elements));
                        } else {
                            handle.set_result(Err(Box::new(AppError::new(
                                "Erreur dans la récupération du contenu de la clé",
                            ))));
                        }
                    } else {
                        handle.set_result(Err(Box::new(AppError::new(
                            "Le serveur a retourné un pb, la clé n'existe peut être pas",
                        ))));
                    }
                } else {
                    handle.set_result(Err(Box::new(AppError::new(
                        "Erreur dans le lancement de la requete",
                    ))));
                }
            }
        });
    }
}

impl eframe::App for EncrypterApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            label: _,
            value: _,
            selected: _,
            files_folder: _,
            db: _,
            last_message: _,
            is_error: _,
            is_add_opened: _,
            key_sha1_input: _,
            key_name: _,
            flower: _,
        } = self;

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel")
            .exact_width(500.0)
            .show(ctx, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    StripBuilder::new(ui)
                        .size(Size::remainder())
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                if let Err(e) =
                                    EncrypterApp::display_tree(&mut self.files_folder, ui)
                                {
                                    error!("error in display tree: {}", e);
                                }
                            });
                        });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Clés de chiffrage")
                    .selected_text(format!("{:?}", self.selected))
                    .show_ui(ui, |ui| {
                        let keys = self.db.get_all().expect("fail to get keys");
                        for k in keys.iter() {
                            ui.selectable_value(&mut self.selected, Some(k.clone()), &k.name);
                        }
                    });
                if ui.button("Ajouter ..").clicked() {
                    self.is_add_opened = true;
                }
            });

            ui.separator();

            egui::ScrollArea::both().show(ui, |ui| {
                ui.group(|ui| {
                    ui.label("Liste des fichiers sélectionnés :");
                    ui.separator();
                    EncrypterApp::construct_list(&self.files_folder, ui);
                });

                let button_crypt = egui::Button::new(
                    RichText::new("Chiffrer les fichiers sélectionnés").color(Color32::BLUE),
                );
                if let Some(selected_key) = &self.selected {
                    if ui.add(button_crypt).clicked() {
                        info!("Chiffrage des fichiers");

                        if let Some(kvalue) = &selected_key.public_key {
                            match EncrypterApp::crypt_selected(
                                &self.files_folder,
                                &selected_key.name,
                                &selected_key.sha1,
                                kvalue,
                            ) {
                                Ok(_) => {
                                    println!("Fin du chiffrage des fichiers");
                                    self.last_message = "Fichiers chiffrés avec succès".into();
                                    self.is_error = false;
                                }
                                Err(e) => {
                                    self.last_message = "Erreur dans le chiffrage".into();
                                    self.is_error = true;
                                    error!("Error in crypt : {:?}", e);
                                }
                            };
                        } else {
                            self.last_message = "no public key".into();
                            self.is_error = true;
                        }
                    }
                } else {
                    // ui.set_enabled(false);
                    // ui.add(button_crypt)
                    //    .on_hover_text("Sélectionnez une clé de cryptage");
                }

                if !&self.last_message.is_empty() {
                    let mut rt = RichText::new(&self.last_message);
                    if self.is_error {
                        rt = rt.color(Color32::RED);
                    }
                    ui.label(rt);
                }
            });

            egui::warn_if_debug_build(ui);
        });

        if self.is_add_opened {
            let f = &self.flower;
            egui::Window::new("Ajouter une carte").show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Sha1 clé");
                        ui.text_edit_singleline(&mut self.key_sha1_input);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Nom de la clé");
                        ui.text_edit_singleline(&mut self.key_name);
                    });
                });

                ui.horizontal(|ui| {
                    if ui.button("Ajouter").clicked() {
                        println!("Ajout de la carte dans la liste");

                        EncrypterApp::download_key(f, self.key_sha1_input.clone());
                    }
                    if ui.button("Annuler").clicked() {
                        self.is_add_opened = false;
                    }

                    // handling result
                    if f.is_active() {
                        f.extract(|r| println!("{}", r)).finalize(|result| {
                            match result {
                                Ok(value) => {
                                    println!("success dans la récupération des clés : {:?}", value);
                                    self.last_message =
                                        "clé ".to_string() + &self.key_sha1_input + " récupérée";
                                    self.is_error = false;

                                    let new_key = Key {
                                        rowid: 0,
                                        name: self.key_name.clone(),
                                        sha1: self.key_sha1_input.clone(),
                                        public_key: Some(value.as_bytes().to_vec()),
                                    };

                                    if let Ok(_r) = self.db.insert(&new_key) {
                                        self.last_message = "clé ".to_string()
                                            + &self.key_sha1_input
                                            + " récupérée, et sauvegardées";
                                        self.is_error = false;
                                    } else {
                                        self.last_message = "clé ".to_string()
                                            + &self.key_sha1_input
                                            + " non sauvegardée, erreur dans l'écriture";
                                        self.is_error = true;
                                    }

                                    self.is_add_opened = false;
                                }
                                Err(Cause::Suppose(msg)) => {
                                    println!("{}", msg);
                                    self.last_message = msg;
                                    self.is_error = true;
                                }
                                Err(Cause::Panicked(_msg)) => {
                                    // Handle things if stuff unexpectedly panicked at runtime.
                                    self.last_message = _msg;
                                    self.is_error = true;
                                }
                            }
                        });
                    }
                });
            });
        }
    }
}
