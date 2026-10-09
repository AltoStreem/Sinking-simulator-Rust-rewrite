//! GUI.java reset operations using Main's retained native ship references.
#![allow(dead_code)]
use crate::{
    camera_control::SourceCameraControl,
    gui::ResetShipOperations,
    main_globals,
    ship_thumbnail::ShipThumbnail,
    source_ship::{ShipSceneStateBackend, SourceShip, TextureResolver},
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};
pub(crate) struct SourceResetShipOperations {
    pub constructor: Box<
        dyn FnMut(
            Rc<ShipThumbnail>,
            Rc<RefCell<SourceCameraControl>>,
        ) -> Result<main_globals::Ship, String>,
    >,
}
impl SourceResetShipOperations {
    #[allow(clippy::too_many_arguments)]
    pub fn native(
        environment: crate::passes::native_pass_factory::NativePassEnvironment,
        textures: Arc<Mutex<dyn crate::texture::TextureBackend>>,
        shaders: Arc<crate::ship_shaders::SourceShipShaders>,
        struts: Arc<crate::ship_struts::SourceShipStrutShader>,
        resolver: TextureResolver,
        state: Rc<RefCell<dyn ShipSceneStateBackend>>,
        clock: Rc<dyn Fn() -> i64>,
    ) -> Self {
        Self {
            constructor: Box::new(move |thumbnail, control| {
                let clock = clock.clone();
                SourceShip::from_thumbnail_current(
                    thumbnail,
                    main_globals::get_global_materials,
                    control,
                    environment.clone(),
                    textures.clone(),
                    shaders.clone(),
                    struts.clone(),
                    resolver.clone(),
                    state.clone(),
                    Box::new(move || clock()),
                )
                .map(|ship| Rc::new(RefCell::new(ship)))
            }),
        }
    }
}
impl ResetShipOperations for SourceResetShipOperations {
    type Ship = RefCell<SourceShip>;
    type Thumbnail = Rc<ShipThumbnail>;
    fn global_ship(&mut self) -> main_globals::Ship {
        main_globals::get_global_ship()
    }
    fn thumbnail(&mut self, ship: &Self::Ship) -> Self::Thumbnail {
        ship.borrow().thumbnail.clone()
    }
    fn close_ship(&mut self, ship: &Self::Ship) {
        ship.borrow().close();
    }
    fn construct_ship(
        &mut self,
        thumbnail: Self::Thumbnail,
        control: Rc<RefCell<SourceCameraControl>>,
    ) -> main_globals::Ship {
        (self.constructor)(thumbnail, control).unwrap_or_else(|e| panic!("{e}"))
    }
    fn set_global_ship(&mut self, ship: main_globals::Ship) {
        main_globals::set_global_ship(ship);
    }
    fn clear_ship_list(&mut self) {
        main_globals::get_ship_list().borrow_mut().clear();
    }
    fn add_ship(&mut self, ship: main_globals::Ship) {
        main_globals::get_ship_list().borrow_mut().push(ship);
    }
}
