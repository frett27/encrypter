use crate::folder::*;
use egui::{style::Spacing, RichText, Ui, Vec2, Context};
use egui_extras::{Size, StripBuilder};

#[derive(PartialEq, Clone, Debug)]
pub enum MyEnum {
    First,
    Second,
    Third,
}

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
    selected: MyEnum,

    #[serde(skip)]
    files_folder: FolderNode,
}

impl Default for EncrypterApp {
    fn default() -> Self {
        let mut r = FolderNode {
            expanded: false,
            is_folder: true,
            path: ".".into(),
            subfolders: vec![],
            selected:false,
        };

        // expand the first level
        expand(&mut r);

        Self {
            // Example stuff:
            label: "Encrypter".to_owned(),
            value: 2.7,
            selected: MyEnum::Second,
            files_folder: r,
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

        use eframe::egui::{Style, Visuals};

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

    fn display_tree(files_folder: &mut FolderNode, ui: &mut Ui) {
        for ele in &mut files_folder.subfolders {
            let element_name = ele.name();
            if (ele.is_folder) {
                let r = ui.collapsing(element_name, |ui| {
                    // ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    EncrypterApp::display_tree(ele, ui);
                   
                });
                if r.fully_open() {
                    if !ele.expanded {
                        expand(ele);
                    }
                }
            } else {
                ui.checkbox(&mut ele.selected, element_name);
                // ui.label(ele.name());
            }
        }
    }

    fn construct_list(file_folder: &FolderNode, ui: &mut Ui) {
        if file_folder.selected {
            ui.label(file_folder.clone().name());
           
        }
        for elem in &file_folder.subfolders {
            EncrypterApp::construct_list(&elem, ui);
        }
    }

}

impl eframe::App for EncrypterApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            label,
            value,
            selected: MyEnum,
            files_folder: FolderNode,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

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
            // ui.heading("Files");

            egui::ScrollArea::both()
            .show(ui, |ui| {

                StripBuilder::new(ui)
                .size(Size::remainder()).horizontal(|mut strip| {
                    strip.cell(|ui| {
                        EncrypterApp::display_tree(&mut self.files_folder, ui);
                    });
                });

            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("essai");
                if (ui.button(RichText::new("delete")).drag_started()) {
                    ui.label("clicked");
                }
            });

            egui::ComboBox::from_label("Select one!")
                .selected_text(format!("{:?}", self.selected))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected, MyEnum::First, "First");
                    ui.selectable_value(&mut self.selected, MyEnum::Second, "Second");
                    ui.selectable_value(&mut self.selected, MyEnum::Third, "Third");
                });

                EncrypterApp::construct_list(&self.files_folder, ui);

            egui::warn_if_debug_build(ui);
        });

    }
}
