use super::ship_resource::ShipResource;
use std::{ops::Deref, path::PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FileShipResource {
    resource: ShipResource,
    pub(crate) path: PathBuf,
}

impl FileShipResource {
    pub(crate) fn image_data(&self) -> Result<image::RgbaImage, String> {
        if self.resource_type == super::resource_type::ResourceType::Materials {
            return Err(format!(
                "{} is a material palette, not an image",
                self.path.display()
            ));
        }
        crate::file_reader::FileReader::game()
            .read_image(&self.path, 4)
            .and_then(|image| image.into_rgba())
            .map_err(|error| format!("{}: {error}", self.path.display()))
    }

    pub(crate) fn materials(&self) -> Result<crate::materials::Materials, String> {
        if self.resource_type != super::resource_type::ResourceType::Materials {
            return Err(format!("{} is not a material palette", self.path.display()));
        }
        let json = crate::file_reader::FileReader::game()
            .read_file(&self.path)
            .map_err(|error| format!("{}: {error}", self.path.display()))?;
        crate::materials::Materials::from_json(&json)
    }

    pub(crate) fn new(path: PathBuf, resource: ShipResource) -> Result<Self, String> {
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if !extension.eq_ignore_ascii_case(resource.resource_type.extension()) {
            return Err(format!(
                "resource {} has .{extension}; {} resources require .{}",
                path.display(),
                resource.resource_type.name(),
                resource.resource_type.extension()
            ));
        }
        Ok(Self { resource, path })
    }
}

impl Deref for FileShipResource {
    type Target = ShipResource;
    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}
