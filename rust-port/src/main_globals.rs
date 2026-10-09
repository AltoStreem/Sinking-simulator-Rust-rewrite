//! Main.java's retained static fields and property accessors.
//! The source runtime thread owns these objects; JVM cross-thread visibility is pending.
#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc, sync::Arc};
pub(crate) type Ship = Rc<RefCell<crate::source_ship::SourceShip>>;
pub(crate) type Player = Rc<RefCell<crate::music_player::SourceMusicPlayer>>;
pub(crate) type ShipList = Rc<RefCell<Vec<Ship>>>;
#[derive(Default)]
pub(crate) struct MainGlobals {
    pub global_ship: Option<Ship>,
    pub global_materials: Option<Arc<crate::materials::SourceMaterials>>,
    pub player: Option<Player>,
    ship_list: ShipList,
}
impl MainGlobals {
    pub fn global_ship(&self) -> Ship {
        self.global_ship
            .clone()
            .unwrap_or_else(|| panic!("lateinit property globalShip has not been initialized"))
    }
    pub fn global_materials(&self) -> Arc<crate::materials::SourceMaterials> {
        self.global_materials
            .clone()
            .unwrap_or_else(|| panic!("lateinit property globalMaterials has not been initialized"))
    }
    pub fn player(&self) -> Player {
        self.player
            .clone()
            .unwrap_or_else(|| panic!("lateinit property player has not been initialized"))
    }
    pub fn ship_list(&self) -> ShipList {
        self.ship_list.clone()
    }
}
thread_local! {static GLOBALS:RefCell<MainGlobals>=RefCell::new(MainGlobals::default());}
// Image loader workers must see the same Main.globalMaterials reference.
static MATERIALS: std::sync::Mutex<Option<Arc<crate::materials::SourceMaterials>>> =
    std::sync::Mutex::new(None);
#[cfg(test)]
pub(crate) static MATERIALS_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
pub(crate) fn get_global_ship() -> Ship {
    GLOBALS.with(|g| g.borrow().global_ship())
}
pub(crate) fn set_global_ship(ship: Ship) {
    GLOBALS.with(|g| g.borrow_mut().global_ship = Some(ship));
}
pub(crate) fn get_global_materials() -> Arc<crate::materials::SourceMaterials> {
    let value = MATERIALS.lock().unwrap_or_else(|e| e.into_inner()).clone();
    value.unwrap_or_else(|| panic!("lateinit property globalMaterials has not been initialized"))
}
pub(crate) fn set_global_materials(materials: Arc<crate::materials::SourceMaterials>) {
    *MATERIALS.lock().unwrap_or_else(|e| e.into_inner()) = Some(materials);
}
pub(crate) fn get_player() -> Player {
    GLOBALS.with(|g| g.borrow().player())
}
pub(crate) fn initialize_player(player: Player) {
    GLOBALS.with(|g| g.borrow_mut().player = Some(player));
}
pub(crate) fn get_ship_list() -> ShipList {
    GLOBALS.with(|g| g.borrow().ship_list())
}

/// Test-only startup equivalent; production startup remains explicit in game_app.
#[cfg(test)]
pub(crate) fn fixture_materials() -> std::sync::MutexGuard<'static, ()> {
    let serial = MATERIALS_TEST_LOCK.lock().unwrap();
    set_global_materials(Arc::new(crate::materials::SourceMaterials::from_file(
        std::path::Path::new("assets/config/materials.json"),
        |error| panic!("{error}"),
    )));
    serial
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_fields_begin_uninitialized_and_ship_list_is_retained() {
        let _serial = MATERIALS_TEST_LOCK.lock().unwrap();
        let globals = MainGlobals::default();
        for (name, call) in [
            (
                "globalShip",
                Box::new(|| {
                    globals.global_ship();
                }) as Box<dyn Fn()>,
            ),
            (
                "globalMaterials",
                Box::new(|| {
                    globals.global_materials();
                }),
            ),
            (
                "player",
                Box::new(|| {
                    globals.player();
                }),
            ),
        ] {
            let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(call)).unwrap_err();
            let message = error
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| error.downcast_ref::<&str>().copied())
                .unwrap();
            assert_eq!(
                message,
                format!("lateinit property {name} has not been initialized")
            );
        }
        assert!(globals.ship_list().borrow().is_empty());
        assert!(Rc::ptr_eq(&globals.ship_list(), &globals.ship_list()));
        assert!(Rc::ptr_eq(&get_ship_list(), &get_ship_list()));
        let old = Arc::new(crate::materials::SourceMaterials::default());
        set_global_materials(old.clone());
        assert!(Arc::ptr_eq(&old, &get_global_materials()));
        let new = Arc::new(crate::materials::SourceMaterials::default());
        set_global_materials(new.clone());
        assert!(Arc::ptr_eq(&new, &get_global_materials()));
        assert!(!Arc::ptr_eq(&old, &get_global_materials()));
        let worker = std::thread::spawn(get_global_materials).join().unwrap();
        assert!(Arc::ptr_eq(&new, &worker));
    }
}
