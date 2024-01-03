pub struct I18NMessages {
    pub file: String,
    pub open_folder: String,
}

pub fn create_french_messages() -> I18NMessages {
    I18NMessages {
        file: "Fichier".into(),
        open_folder: "Ouvrir un répertoire ...".into(),
    }
}
