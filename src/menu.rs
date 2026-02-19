use inquire::Select;
use std::fmt;

pub enum MainMenu {
    Loads,
    Maintenance,
    Exit,
}
pub enum MaintMenu {
    RecreateDatabase,
    CheckUpdates,
    Back,
}

pub enum LoadMenu {
    LoadRefs,
    LoadBrands,
    LoadModels,
    LoadYears,
    Back,
}

impl fmt::Display for MainMenu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MainMenu::Loads => write!(f, "📥 Loads"),
            MainMenu::Maintenance => write!(f, "🛠️  Maintenance"),
            MainMenu::Exit => write!(f, "🔌 Exit"),
        }
    }
}

impl fmt::Display for LoadMenu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoadMenu::LoadRefs => write!(f, "Load References"),
            LoadMenu::LoadBrands => write!(f, "Load Brands"),
            LoadMenu::LoadModels => write!(f, "Load Models"),
            LoadMenu::LoadYears => write!(f, "Load Years"),
            LoadMenu::Back => write!(f, "Back"),
        }
    }
}

impl fmt::Display for MaintMenu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaintMenu::RecreateDatabase => write!(f, "Recreate Database"),
            MaintMenu::CheckUpdates => write!(f, "Check for Updates"),
            MaintMenu::Back => write!(f, "Back"),
        }
    }
}
