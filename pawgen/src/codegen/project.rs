use std::{
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::schema;

use super::SpriteBuilder;

/// Builder for scratch projects.
/// This handles everything from json generation to asset bundling.
pub struct ProjectBuilder {
    pub(crate) project: super::ProjectCell,
    pub(crate) assets: Vec<ProjectAsset>,
    pub(crate) stage_sprite: Option<SpriteBuilder>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectAsset {
    pub(super) name: String,
    pub(super) data_format: String,
    pub(super) hash: String,
    pub(super) md5ext: String,
    pub(super) source: PathBuf,
}

impl Into<schema::Costume> for &ProjectAsset {
    fn into(self) -> schema::Costume {
        // TODO: Read file and fetch more data from It.
        schema::Costume {
            name: self.name.clone(),
            bitmap_resolution: 1,
            data_format: self.data_format.clone(),
            asset_id: self.hash.clone(),
            md5ext: self.md5ext.clone(),
            rotation_center_x: 0,
            rotation_center_y: 0,
        }
    }
}

impl Into<schema::Sound> for &ProjectAsset {
    fn into(self) -> schema::Sound {
        // TODO: Read file and fetch more data from It.
        schema::Sound {
            name: self.name.clone(),
            asset_id: self.hash.clone(),
            data_format: self.data_format.clone(),
            format: String::new(),
            rate: 48000,
            sample_count: 40681,
            md5ext: self.md5ext.clone(),
        }
    }
}

impl ProjectBuilder {
    pub fn new() -> Self {
        Self {
            project: super::ProjectCell::default(),
            assets: Vec::new(),
            stage_sprite: None,
        }
    }

    /// This method crates basic project data including for example
    /// stage sprite.
    pub fn init_core(&mut self) {
        let stage = self.create_sprite("Stage");
        stage.set_stage(true);

        self.stage_sprite = Some(stage);
    }

    pub fn get_stage(&mut self) -> &mut SpriteBuilder {
        self.stage_sprite.as_mut().unwrap()
    }

    /// Adds asset to the project. This precomputes asset's hash and
    /// ensures It is included in the project bundle.
    pub fn register_asset(
        &mut self,
        name: impl AsRef<str>,
        source: impl AsRef<Path>,
    ) -> std::io::Result<ProjectAsset> {
        let file = std::fs::read(source.as_ref())?;
        let hash = hex::encode(&md5::compute(file).0[..]);
        let data_format = source.as_ref().extension().unwrap().to_str().unwrap();
        let md5ext = format!("{hash}.{data_format}");

        let asset = ProjectAsset {
            name: name.as_ref().to_owned(),
            data_format: data_format.to_string(),
            hash,
            md5ext,
            source: source.as_ref().to_owned(),
        };

        self.assets.push(asset.clone());
        Ok(asset)
    }

    pub fn add_extension(&self, extension: impl AsRef<str>) {
        self.project
            .borrow_mut()
            .extensions
            .insert(extension.as_ref().to_owned());
    }

    pub fn create_sprite(&self, name: impl AsRef<str>) -> SpriteBuilder {
        self.project
            .borrow_mut()
            .targets
            .push(schema::ProjectTarget {
                name: name.as_ref().to_owned(),
                volume: 100,
                layer_order: 1,
                size: 100,
                direction: 90,
                draggable: false,
                rotation_style: "all around".to_string(),
                visible: true,
                ..Default::default()
            });

        SpriteBuilder::new(
            self.project.clone(),
            self.project.borrow().targets.len() - 1,
        )
    }

    pub fn bundle_project(self, output: impl AsRef<Path>) -> std::io::Result<()> {
        use zip::{write::SimpleFileOptions, ZipWriter};

        let zip_options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

        let target_writer = std::fs::File::create(output.as_ref())?;
        let mut zip = ZipWriter::new(BufWriter::new(target_writer));

        zip.start_file("project.json", zip_options)?;
        let project = self.project.borrow();
        zip.write_all(&serde_json::ser::to_vec(&*project)?)?;

        for asset in self.assets.iter() {
            zip.start_file(&asset.md5ext, zip_options)?;
            zip.write_all(&std::fs::read(&asset.source)?)?;
        }

        zip.finish()?;

        Ok(())
    }
}
