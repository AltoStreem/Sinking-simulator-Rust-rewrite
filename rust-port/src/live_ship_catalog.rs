//! Engine-thread adapter for the original Toolbox reload/thumbnail lifetime.
use crate::*;
use std::{path::{Path,PathBuf},rc::Rc};
use toolbox_reload_filesystem::{FilesystemReloadBackend,LoadedThumbnail,SourceFile,SteamFolderAccess,strip_verbatim_prefix};
struct ExcludedSteam;
impl SteamFolderAccess for ExcludedSteam {
    fn is_running(&mut self)->bool {false}
    fn all_workshop_folders(&mut self)->Vec<PathBuf> {panic!("Steam is excluded; workshop access must never be invoked")}
}
pub(crate) struct LiveCatalog {
    pub toolbox:toolbox::SourceToolbox,
    backend:FilesystemReloadBackend,
    pub revision:u64,
    elapsed:std::time::Duration,
    asset_root:PathBuf,
}
#[derive(Resource,Default)]
pub(crate) struct PendingSelection(pub Option<ship_thumbnail::ShipThumbnail>);
impl LiveCatalog {
    pub(crate) fn new(asset_root:PathBuf,ships:PathBuf)->Self {
        let mut catalog=Self {toolbox:toolbox::SourceToolbox::default(),backend:FilesystemReloadBackend::with_local_root(asset_root.clone(),ships,Box::new(ExcludedSteam)),revision:0,elapsed:std::time::Duration::ZERO,asset_root};
        // Original coroutine executes one scan immediately, then delay(1000).
        catalog.reload();catalog
    }
    pub(crate) fn extracted()->Self {
        let assets=std::env::current_dir().expect("game working directory").join("assets");
        Self::new(assets.clone(),assets.join("source_ships"))
    }
    pub(crate) fn reload(&mut self) {
        if let Err(error)=self.toolbox.reload_files(&mut self.backend) {bevy::log::error!("Source ship catalog reload failed: {error}");return;}
        self.revision=self.revision.wrapping_add(1);
    }
    pub(crate) fn tick(&mut self,delta:std::time::Duration) {
        self.elapsed=self.elapsed.saturating_add(delta);
        if self.elapsed>=std::time::Duration::from_secs(1) {
            // Source delay follows each completed scan, not a catch-up burst.
            self.elapsed=std::time::Duration::ZERO;self.reload();
        }
    }
    fn key(&self,path:&Path)->SourceFile {
        let path=if path.is_absolute(){path.to_path_buf()}else{self.asset_root.join(path)};
        Self::canonical_key(&path)
    }
    fn canonical_key(path:&Path)->SourceFile {
        let canonical=std::fs::canonicalize(path).or_else(|_| {
            let parent=path.parent().ok_or_else(||std::io::Error::other("missing parent"))?;
            std::fs::canonicalize(parent).map(|parent|parent.join(path.file_name().unwrap_or_default()))
        }).unwrap_or_else(|_|path.to_path_buf());
        SourceFile::new(strip_verbatim_prefix(canonical))
    }
    pub(crate) fn active_index(&self,ui:&ShipCatalog,active:&ship_thumbnail::ShipThumbnail)->Option<usize> {
        let Some(ship_thumbnail::ThumbnailResource::File(base))=active.get_resource(ShipResourceType::Base,&ShipLayer::default()) else {return None;};
        // Initial Main uses a relative file path; Toolbox uses canonical files.
        // Compare file identities while keeping those resource objects independent.
        let key=Self::canonical_key(&base.path);
        ui.0.iter().position(|choice|self.key(Path::new(&choice.physics_asset))==key)
    }
    pub(crate) fn loaded(&self,choice:&ShipChoice)->Option<Rc<LoadedThumbnail>> {
        let key=self.key(Path::new(&choice.physics_asset));
        self.toolbox.catalog().ship_thumbnails.iter().find(|thumbnail| {
            match thumbnail.translated.get_resource(ShipResourceType::Base,&ShipLayer::default()) {
                Some(ship_thumbnail::ThumbnailResource::File(file))=>SourceFile::new(file.path.clone())==key,
                _=>false,
            }
        }).cloned()
    }
    /// Publish the exact translated old-map thumbnail list, including its Java name order.
    pub(crate) fn ui_catalog(&self)->ShipCatalog {
        let mut choices=Vec::new();let mut layers=Vec::new();
        let relative=|path:&Path|path.strip_prefix(&self.asset_root).unwrap_or(path).to_string_lossy().replace('\\',"/");
        for loaded in &self.toolbox.catalog().ship_thumbnails {
            let thumbnail=&loaded.translated;
            let Some(ship_thumbnail::ThumbnailResource::File(base))=thumbnail.get_resource(ShipResourceType::Base,&ShipLayer::default()) else {continue;};
            let path=|resource:&ship_thumbnail::ThumbnailResource|match resource {
                ship_thumbnail::ThumbnailResource::File(file)=>relative(&file.path),
                ship_thumbnail::ThumbnailResource::BaseDerivedTexture(derived)=>relative(&derived.base_resource.path),
            };
            let asset=thumbnail.get_resource(ShipResourceType::Texture,&ShipLayer::default()).map(&path).unwrap_or_else(||relative(&base.path));
            choices.push(ShipChoice {name:thumbnail.name().unwrap_or(&base.ship).to_owned(),asset,physics_asset:relative(&base.path),material_map:true,scale:0.36,source_key:Some(SourceShipKey {directory:base.path.parent().unwrap_or(Path::new("")).to_path_buf(),ship:base.ship.clone()})});
            layers.push(thumbnail.ordered_layers().iter().filter_map(|layer|thumbnail.get_resource(ShipResourceType::Texture,layer).map(|resource|ShipLayerChoice {name:layer.clone(),asset:path(resource)})).collect());
        }
        ShipCatalog(choices,layers)
    }
    pub(crate) fn thumbnail(&self,choice:&ShipChoice)->Option<ship_thumbnail::ShipThumbnail> {
        self.loaded(choice).map(|loaded|loaded.translated.clone())
    }
}
pub(crate) fn advance(time:Res<Time<Real>>,mut catalog:NonSendMut<LiveCatalog>) {catalog.tick(time.delta());}


#[cfg(test)]
mod tests {
    use super::*;
    struct Folder(PathBuf);
    impl Folder {
        fn new()->Self {
            let path=std::env::temp_dir().join(format!("ss2_live_catalog_{}_{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            std::fs::create_dir(&path).unwrap();Self(path)
        }
        fn image(&self,name:&str,width:u32)->PathBuf {
            let file=self.0.join(name);
            image::RgbaImage::from_pixel(width,2,image::Rgba([0x5d,0x5d,0x60,255])).save(&file).unwrap();file
        }
    }
    impl Drop for Folder {
        fn drop(&mut self) {
            assert!(self.0.starts_with(std::env::temp_dir()));
            for entry in std::fs::read_dir(&self.0).unwrap() {std::fs::remove_file(entry.unwrap().path()).unwrap();}
            std::fs::remove_dir(&self.0).unwrap();
        }
    }
    #[test]
    fn independent_main_relative_thumbnail_resolves_to_canonical_browser_index_without_sharing_its_resource() {
        let mut live=LiveCatalog::extracted();live.reload();let ui=live.ui_catalog();assert!(!ui.0.is_empty());
        let relative=Path::new("assets").join(&ui.0[0].physics_asset);
        let main=ship_thumbnail::ShipThumbnail::from_base_file(&relative).unwrap();
        assert_eq!(live.active_index(&ui,&main),Some(0));
        let browser=live.thumbnail(&ui.0[0]).unwrap();
        assert_ne!(main.get_resource(ShipResourceType::Base,&ShipLayer::default()),browser.get_resource(ShipResourceType::Base,&ShipLayer::default()),"Main startup and Toolbox create independent original resources");
    }
    #[test]
    fn live_catalog_publishes_old_map_java_order_and_retains_cached_resources_until_two_scans_after_removal() {
        let _serial=main_globals::fixture_materials();
        let folder=Folder::new();let a=folder.image("Alpha_BASE.png",2);folder.image("Zeta_BASE.png",4);
        let mut live=LiveCatalog::new(folder.0.clone(),folder.0.clone());
        assert_eq!(live.revision,1);assert!(live.ui_catalog().0.is_empty());
        live.tick(std::time::Duration::from_millis(999));assert_eq!(live.revision,1);
        live.tick(std::time::Duration::from_millis(1));assert_eq!(live.revision,2);
        let ui=live.ui_catalog();assert_eq!(ui.0.iter().map(|choice|choice.name.as_str()).collect::<Vec<_>>(),vec!["Alpha","Zeta"]);
        let old=live.loaded(&ui.0[0]).unwrap();let active=old.translated.clone();
        let original=ShipStructure::load_for_thumbnail(&active).unwrap();assert_eq!(original.texel_width,2);
        let mut images=Assets::<Image>::default();
        let handle=ship_upload_preview::texture(&active,ShipResourceType::Texture,&ShipLayer::default(),&mut images).unwrap().unwrap();
        std::fs::write(&a,b"modified on disk after source image cache populated").unwrap();
        live.tick(std::time::Duration::from_secs(9));
        assert_eq!(live.revision,3,"Long frame performs one scan; original delay has no catch-up burst");
        assert!(Rc::ptr_eq(&old,&live.loaded(&ui.0[0]).unwrap()));
        let polled=match live.thumbnail(&ui.0[0]).unwrap().get_resource(ShipResourceType::Texture,&ShipLayer::default()).unwrap() {
            ship_thumbnail::ThumbnailResource::BaseDerivedTexture(resource)=>resource.texture_current(&mut images).unwrap().unwrap(),
            _=>panic!("Expected synthesized source texture"),
        };
        assert_eq!(polled,handle,"Browser poll and selected ship share the retained texture cache");
        assert_eq!(ShipStructure::load_for_thumbnail(&live.thumbnail(&ui.0[0]).unwrap()).unwrap().texel_width,2);
        std::fs::remove_file(a).unwrap();folder.image("Beta_BASE.png",3);
        live.reload();assert_eq!(live.ui_catalog().0.iter().map(|c|c.name.as_str()).collect::<Vec<_>>(),vec!["Alpha","Zeta"]);
        live.reload();assert_eq!(live.ui_catalog().0.iter().map(|c|c.name.as_str()).collect::<Vec<_>>(),vec!["Beta","Zeta"]);
        assert!(live.loaded(&ui.0[0]).is_none());
        assert_eq!(ShipStructure::load_for_thumbnail(&active).unwrap().texel_width,2,"Active ship is independent of the browser list lifetime");
        assert_eq!(ship_upload_preview::texture(&active,ShipResourceType::Texture,&ShipLayer::default(),&mut images).unwrap().unwrap(),handle);
    }

    #[test]
    fn actual_browser_poll_click_same_ship_reset_and_removed_catalog_preserve_source_cache_and_active_layers() {
        let _serial=main_globals::fixture_materials();let folder=Folder::new();
        folder.image("Alpha_BASE.png",2);let b=folder.image("Beta_BASE.png",5);folder.image("Beta_exterior_TEXTURE.png",5);
        let mut live=LiveCatalog::new(folder.0.clone(),folder.0.clone());live.reload();
        let ui=live.ui_catalog();let old=live.thumbnail(&ui.0[0]).unwrap();
        let structure=ShipStructure::load_for_thumbnail(&old).unwrap();
        let mut buffers=Assets::<ShaderBuffer>::default();let gpu=make_gpu_ship_physics_assets(&structure,&mut buffers);
        let mut app=App::new();
        app.add_plugins((MinimalPlugins,AssetPlugin::default())).init_asset::<Image>();
        app.insert_non_send_resource(live).insert_resource(ui).insert_resource(structure).insert_resource(buffers).insert_resource(gpu)
            .insert_resource(ship_runtime_reset::ActiveThumbnail(Some(old)))
            .init_resource::<Simulation>().init_resource::<ship_browser_ui::BrowserLayout>()
            .init_resource::<Assets<Image>>().init_resource::<Assets<Mesh>>()
            .init_resource::<ship_upload_preview::ActivePreview>().init_resource::<PendingSelection>()
            .init_resource::<tools::move_tool::MoveDragState>().init_resource::<GpuShipPhysicsSnapshot>()
            .init_resource::<ButtonInput<MouseButton>>().init_resource::<ButtonInput<KeyCode>>()
            .add_message::<MouseWheel>().add_message::<window_characters::ResetShip>()
            .add_systems(Update,(refresh_ship_catalog,ship_browser_ui::input,load_selected_ship,ship_browser_ui::sync).chain());
        let window=app.world_mut().spawn(Window {resolution:(1280,720).into(),..default()}).id();
        let image=app.world_mut().spawn((ShipThumbnail(1),Sprite::default(),Transform::default(),Visibility::Hidden)).id();
        app.update();
        assert!(app.world().resource::<ship_browser_ui::BrowserLayout>().rows.is_empty(),"First nonblocking getTexture schedules a worker without forcing a synchronous load");
        let start=std::time::Instant::now();
        while app.world().resource::<ship_browser_ui::BrowserLayout>().rows.len()!=2 {
            assert!(start.elapsed()<std::time::Duration::from_secs(5));std::thread::sleep(std::time::Duration::from_millis(5));app.update();
        }
        let visible=app.world().get::<Sprite>(image).unwrap().image.clone();
        // Loading BASE in the synthesized preview populates the same selected ship image cache.
        std::fs::write(&b,b"BASE overwritten after browser cache was loaded").unwrap();
        let row=app.world().resource::<ship_browser_ui::BrowserLayout>().rows.iter().find(|row|row.0==1).unwrap().1;
        app.world_mut().get_mut::<Window>(window).unwrap().set_cursor_position(Some(Vec2::new(640.0+row.center().x,360.0-row.center().y)));
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        // Source native character callbacks precede the Toolbox GUI selection.
        app.world_mut().write_message(window_characters::ResetShip);app.world_mut().write_message(window_characters::ResetShip);
        let generation=app.world().resource::<GpuShipPhysicsAssets>().generation;app.update();
        assert_eq!(app.world().resource::<GpuShipPhysicsAssets>().generation,generation+3,"Two resets then one source browser construction");
        assert_eq!(app.world().resource::<ShipStructure>().texel_width,5);
        assert_eq!(app.world().resource::<Simulation>().selected_layer,0);
        let retained=app.world().resource::<ship_runtime_reset::ActiveThumbnail>();let thumbnail=retained.0.as_ref().unwrap().clone();
        assert_eq!(ship_upload_preview::layer_count(app.world().resource::<ShipCatalog>(),app.world().resource::<Simulation>(),None,Some(retained)),2);
        let selected=ship_upload_preview::texture(&thumbnail,ShipResourceType::Texture,&ShipLayer::default(),&mut app.world_mut().resource_mut::<Assets<Image>>()).unwrap().unwrap();
        assert_eq!(selected,visible,"Rendered browser texture and selected ship use one source resource cache");
        // A second click on the identical row still constructs a fresh Ship.
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().reset_all();app.update();
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();assert_eq!(app.world().resource::<GpuShipPhysicsAssets>().generation,generation+4);
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().reset_all();
        // Drop the published browser entry while keeping the selected ship and exterior layer active.
        app.world_mut().resource_mut::<Simulation>().selected_layer=1;
        std::fs::remove_file(b).unwrap();std::fs::remove_file(folder.0.join("Beta_exterior_TEXTURE.png")).unwrap();
        let mut live=app.world_mut().non_send_resource_mut::<LiveCatalog>();live.reload();live.reload();
        app.update();
        assert_eq!(app.world().resource::<ShipCatalog>().0.iter().map(|c|c.name.as_str()).collect::<Vec<_>>(),vec!["Alpha"]);
        assert_eq!(app.world().resource::<Simulation>().ship_index,usize::MAX);
        assert_eq!(app.world().resource::<GpuShipPhysicsAssets>().generation,generation+4,"Catalog publication/index changes do not construct an active ship");
        let retained=app.world().resource::<ship_runtime_reset::ActiveThumbnail>();
        assert_eq!(ship_upload_preview::layer(app.world().resource::<ShipCatalog>(),app.world().resource::<Simulation>(),None,Some(retained),1).unwrap().display_name(),"exterior");
        app.world_mut().write_message(window_characters::ResetShip);app.update();
        assert_eq!(app.world().resource::<ShipStructure>().texel_width,5);assert_eq!(app.world().resource::<GpuShipPhysicsAssets>().generation,generation+5);
    }
}
