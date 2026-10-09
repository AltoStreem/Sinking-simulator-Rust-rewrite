mod base_derived_texture_ship_resource;
mod file_ship_resource;
mod image_cache;
mod layer;
mod layer_companion;
mod resource_type;
mod ship_resource;
mod ship_resource_cleanup_loop;
mod ship_resource_cleanup_predicate;
mod ship_resource_companion;
mod ship_resource_image_data_loader;
mod ship_resource_image_data_now;
mod ship_resource_materials_initializer;
mod source_texture_cache;
mod texture_cache;
mod texture_config;
mod texture_now_config;

pub(crate) use base_derived_texture_ship_resource::BaseDerivedTextureShipResource;
pub(crate) use file_ship_resource::FileShipResource as ShipResourceFile;
pub(crate) use layer::Layer as ShipLayer;
pub(crate) use resource_type::ResourceType as ShipResourceType;
pub(crate) use ship_resource::{ShipResource, parse_resource_path};
pub(crate) use source_texture_cache::SourceTextureEnvironment;

/// Native Ship getters reach the resource's shared texture cache on every call.
pub(crate) fn source_texture_resolver(
    environment: SourceTextureEnvironment,
    global: std::sync::Arc<crate::materials::SourceMaterials>,
) -> crate::source_ship::TextureResolver {
    source_texture_resolver_current(environment, std::sync::Arc::new(move || global.clone()))
}
pub(crate) fn source_texture_resolver_current(
    environment: SourceTextureEnvironment,
    global: base_derived_texture_ship_resource::SourcePalette,
) -> crate::source_ship::TextureResolver {
    std::rc::Rc::new(move |resource| match resource {
        crate::ship_thumbnail::ThumbnailResource::File(file) => {
            file.source_texture_now(&environment).map(Some)
        }
        crate::ship_thumbnail::ThumbnailResource::BaseDerivedTexture(derived) => derived
            .source_texture_now(&environment, global.clone())
            .map(Some),
    })
}
pub(crate) fn source_texture_resolver_globals(
    environment: SourceTextureEnvironment,
) -> crate::source_ship::TextureResolver {
    source_texture_resolver_current(
        environment,
        std::sync::Arc::new(crate::main_globals::get_global_materials),
    )
}
