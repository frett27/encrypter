use egui::Button;
use log::{error, info};

use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use crate::encrypt::check_public_key;
use crate::encrypt::encrypt_file_with_inmemory_key;
use crate::folder;
use crate::folder::*;

use crate::keys_management::*;
use egui::Color32;
use egui::{Context, RichText, Ui, Vec2};
use egui_extras::{Size, StripBuilder};

use flowync::error::Cause;
use flowync::Flower;

use isahc::prelude::*;

use im_native_dialog::ImNativeFileDialog;

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

    // dialog for adding keys
    #[serde(skip)]
    is_add_opened: bool,
    #[serde(skip)]
    key_name: String,
    #[serde(skip)]
    key_sha1_input: String,

    #[serde(skip)]
    key_search_key_internet: bool,
    #[serde(skip)]
    key_public_key: String,

    #[serde(skip)]
    key_error_message: String,
    #[serde(skip)]
    key_is_error: bool,

    // async grab key from internet
    #[serde(skip)]
    flower: TypedFlower,

    #[serde(skip)]
    file_path: PathBuf,

    #[serde(skip)]
    file_path_dialog: ImNativeFileDialog<Option<PathBuf>>,

    #[serde(skip)]
    i18n: crate::i18n::I18NMessages,
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
            key_public_key: "".to_owned(),
            key_search_key_internet: true,
            key_error_message: "".to_owned(),
            key_is_error: false,

            flower: TypedFlower::new(1),
            file_path: PathBuf::from("."),
            file_path_dialog: im_native_dialog::ImNativeFileDialog::default(),
            i18n: crate::i18n::create_french_messages(),
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
                    if ui
                        .button("Selectionner tous les fichiers du répertoire")
                        .clicked()
                    {
                        // handle selection
                        for e in &mut ele.subfolders {
                            if !e.is_folder {
                                e.selected = true;
                            }
                        }
                    }

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

    fn construct_list(file_folder: &mut FolderNode, ui: &mut Ui) {
        if file_folder.selected {
            let name_clone = (*file_folder.name()).to_string().clone();
            ui.checkbox(
                &mut file_folder.selected,
                RichText::new(name_clone).color(Color32::DARK_BLUE),
            );
            //  ui.label();
        }
        for elem in file_folder.subfolders.iter_mut() {
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

                match response_result {
                    Ok(mut response) => {
                        if response.status().is_success() {
                            let text = response.text(); // read response

                            match text {
                                Ok(returned_elements) => {
                                    // Set result and then extract later.
                                    handle.set_result(Ok(returned_elements));
                                }
                                Err(e) => {
                                    handle.set_result(Err(Box::new(AppError::new(format!(
                                        "Erreur dans la récupération du contenu de la clé : {:?}",
                                        e
                                    )))));
                                }
                            }
                        } else {
                            handle.set_result(Err(Box::new(AppError::new(format!(
                "Le serveur a retourné un pb, la clé n'existe peut être pas : {:?} -> {:?}", 
                    response.status(), response.body()
            )))));
                        }
                    }
                    Err(e) => {
                        handle.set_result(Err(Box::new(AppError::new(format!(
                            "Erreur dans le lancement de la requete : {:?}",
                            e
                        )))));
                    }
                }
            }
        });
    }

    fn clean_message(&mut self) {
        self.last_message = "".into();
        self.is_error = false;
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
            key_public_key: _,
            key_error_message: _,
            key_search_key_internet: _,
            key_is_error: _,
            flower: _,
            file_path_dialog: _,
            file_path: _,
            i18n,
        } = self;

        if let Some(result) = self.file_path_dialog.check() {
            match result {
                Ok(Some(path)) => {
                    ctx.request_repaint();
                    self.file_path = path;
                    println!(
                        "selected folder : {}",
                        self.file_path.as_path().to_str().unwrap()
                    );
                    let mut new_folder = FolderNode {
                        expanded: false,
                        is_folder: true,
                        path: self.file_path.as_path().to_str().unwrap().into(),
                        subfolders: vec![],
                        selected: false,
                    };
                    folder::expand(&mut new_folder).expect("hello");
                    // self.files_folder.expand();
                    self.files_folder = new_folder;
                }
                Ok(None) => {}
                Err(error) => {
                    eprintln!("Error selecting xplane_path: {}", error)
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button((i18n.file).to_string(), |ui| {
                    if ui.button(&i18n.open_folder).clicked() {
                        let location = self
                            .file_path
                            .parent()
                            .map(|location| location.to_path_buf());

                        //let repaint_signal = ctx.repaint_signal();
                        self.file_path_dialog
                            //.with_callback(move |_| c.request_repaint())
                            .open_single_dir(location)
                            .expect("Unable to open file_path dialog");
                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });

                ui.menu_button("Clés", |ui| {
                    // ajout de clef
                    if ui.button("Ajouter ou mettre à jour une clé ..").clicked() {
                        self.key_name = "".into();
                        self.key_sha1_input = "".into();
                        self.key_error_message = "".into();
                        self.key_is_error = false;
                        self.key_public_key = "".into();
                        self.key_search_key_internet = true;
                        self.is_add_opened = true;
                    }
                });
            });
        });

        // folder tree display
        egui::SidePanel::left("side_panel")
            .exact_width(500.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("1 - Selectionnez les fichiers à chiffrer");
                    ui.separator();

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
                })
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("2 - Sélectionnez la clé de chiffrement")
                        .on_hover_text(
                            "Si la clef n'existe pas, vous pouvez l'ajouter avec le menu",
                        );
                });
                ui.separator();
                ui.horizontal(|ui| {
                    // selected text
                    let mut selectable_text: String = "".into();
                    if let Some(v) = &self.selected {
                        selectable_text = text_representation(v);
                    }

                    ui.horizontal(|ui| {
                        ui.label("Clé :");

                        let choice_key = egui::ComboBox::from_label("")
                            .selected_text(selectable_text.to_string())
                            .width(800.0)
                            .show_ui(ui, |ui| {
                                let keys = self.db.get_all().expect("fail to get keys");
                                for k in keys.iter() {
                                    ui.selectable_value(
                                        &mut self.selected,
                                        Some(k.clone()),
                                        text_representation(k),
                                    );
                                }
                            });
                        if choice_key.response.changed() {
                            self.clean_message();
                        }
                    });
                });

                ui.group(|ui| {
                    ui.label("Liste des fichiers sélectionnés :");
                    ui.separator();
                    EncrypterApp::construct_list(&mut self.files_folder, ui);
                });

                let button_crypt = egui::Button::new(
                    RichText::new("3 - Chiffrer les fichiers sélectionnés").color(Color32::BLUE),
                );
                if let Some(selected_key) = &self.selected {
                    if ui.add(button_crypt).clicked() {
                        // reset the last_message
                        self.last_message = "".into();
                        self.is_error = false;

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
                                    self.last_message =
                                        format!("Erreur dans le chiffrage : {:?}", &e);
                                    self.is_error = true;
                                    error!("Error in crypt : {:?}", e);
                                }
                            };

                            let mut r = FolderNode {
                                expanded: false,
                                is_folder: true,
                                path: ".".into(),
                                subfolders: vec![],
                                selected: false,
                            };

                            let _ = expand(&mut r);

                            self.files_folder = r;
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

                // display the last message
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
                        ui.label("Sha1 de l'instument / ou de la clé");
                        ui.text_edit_singleline(&mut self.key_sha1_input);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Nom de la clé");
                        ui.text_edit_singleline(&mut self.key_name);
                    });

                    ui.horizontal(|ui| {
                        ui.checkbox(
                            &mut self.key_search_key_internet,
                            "Récupérer la clé sur internet",
                        );
                    });
                    if !self.key_search_key_internet {
                        ui.horizontal(|ui| {
                            ui.label("Clé Publique :");
                            ui.text_edit_multiline(&mut self.key_public_key);
                        });
                    }
                });

                if !self.key_error_message.is_empty() {
                    ui.horizontal(|ui| {
                        let mut rt = RichText::new(&self.key_error_message);
                        if self.key_is_error {
                            rt = rt.color(Color32::RED);
                        }
                        ui.label(rt);
                    });
                }

                let keysrc: String = self.key_sha1_input.trim().into();

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!f.is_active(), Button::new("Ajouter"))
                        .clicked()
                    {
                        println!("Ajout de la carte dans la liste");

                        self.key_error_message = "".into();
                        self.key_is_error = false;

                        if keysrc.trim().len() != 40 {
                            self.key_error_message =
                                "le sha1 de la clé doit avoir 40 characteres".into();
                            self.key_is_error = true;
                            return;
                        }

                        if self.key_search_key_internet {
                            EncrypterApp::download_key(f, self.key_sha1_input.clone());
                        } else {
                            // check the public key,
                            if self.key_public_key.trim().is_empty() {
                                self.key_error_message =
                                    "vous devez saisir une clé publique".into();
                                self.key_is_error = true;
                            } else if let Err(_e) = check_public_key(self.key_public_key.as_bytes())
                            {
                                self.key_error_message =
                                    "la clé publique est invalide, vérifiez la".into();
                                self.key_is_error = true;
                            } else {
                                // ok, record the key into db
                                let new_key = Key {
                                    rowid: 0,
                                    name: self.key_name.clone(),
                                    sha1: keysrc.clone(),
                                    public_key: Some(self.key_public_key.as_bytes().to_vec()),
                                };

                                if let Ok(_r) = self.db.insert(&new_key) {
                                    self.key_error_message =
                                        "clé ".to_string() + &keysrc + " récupérée, et enregistrée";
                                    self.key_is_error = false;
                                } else {
                                    self.key_error_message = "clé ".to_string()
                                        + &keysrc
                                        + " non sauvegardée, erreur dans l'écriture";
                                    self.key_is_error = true;
                                }
                            }
                        }
                    };

                    if ui.button("Fermer").clicked() {
                        self.is_add_opened = false;
                    }

                    // handling result
                    if f.is_active() {
                        f.extract(|r| println!("{}", r)).finalize(|result| {
                            match result {
                                Ok(value) => {
                                    println!("success dans la récupération des clés : {:?}", value);
                                    self.key_error_message =
                                        "clé ".to_string() + &keysrc + " récupérée";
                                    self.key_is_error = false;

                                    let new_key = Key {
                                        rowid: 0,
                                        name: self.key_name.clone(),
                                        sha1: self.key_sha1_input.clone(),
                                        public_key: Some(value.as_bytes().to_vec()),
                                    };

                                    if let Ok(_r) = self.db.insert(&new_key) {
                                        self.key_error_message = "clé ".to_string()
                                            + &self.key_sha1_input
                                            + " récupérée, et sauvegardées";
                                        self.key_is_error = false;
                                    } else {
                                        self.key_error_message = "clé ".to_string()
                                            + &self.key_sha1_input
                                            + " non sauvegardée, erreur dans l'écriture";
                                        self.key_is_error = true;
                                    }
                                }
                                Err(Cause::Suppose(msg)) => {
                                    println!("{}", msg);
                                    self.key_error_message = msg;
                                    self.key_is_error = true;
                                }
                                Err(Cause::Panicked(_msg)) => {
                                    // Handle things if stuff unexpectedly panicked at runtime.
                                    self.key_error_message = _msg;
                                    self.key_is_error = true;
                                }
                            }
                        });
                    }
                });
            });
        }
    }
}
