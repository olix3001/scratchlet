use std::path::{Path, PathBuf};

use super::Procedure;

#[derive(Debug)]
pub struct Sprite {
    pub(super) name: String,
    pub(super) is_stage: bool,
    /// List of this sprite's costumes. First one is default.
    pub(super) costumes: Vec<Costume>,
    pub(super) sounds: Vec<Sound>,

    pub(super) procedures: Vec<Procedure>,
}

impl Sprite {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            is_stage: false,
            costumes: Vec::new(),
            sounds: Vec::new(),
            procedures: Vec::new(),
        }
    }

    pub fn mark_as_stage(&mut self) -> &mut Self {
        self.is_stage = true;
        self
    }

    pub fn add_costume(&mut self, costume: Costume) -> &mut Self {
        self.costumes.push(costume);
        self
    }
    pub fn add_sound(&mut self, sound: Sound) -> &mut Self {
        self.sounds.push(sound);
        self
    }

    pub fn add_procedure(&mut self, procedure: Procedure) -> &mut Self {
        self.procedures.push(procedure);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Costume {
    pub(super) name: String,
    pub(super) source: PathBuf,
}

impl Costume {
    pub fn new(name: impl AsRef<str>, source: impl AsRef<Path>) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            source: source.as_ref().to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sound {
    pub(super) name: String,
    pub(super) source: PathBuf,
}

impl Sound {
    pub fn new(name: impl AsRef<str>, source: impl AsRef<Path>) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            source: source.as_ref().to_owned(),
        }
    }
}
