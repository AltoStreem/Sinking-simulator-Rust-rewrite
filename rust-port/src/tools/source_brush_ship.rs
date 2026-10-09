//! Retained Ship receiver for the original Break/Flood/Dry texture getters.
//! The provider retains its adapter so BrushRuntime's identity comparison
//! matches Main.globalShip replacement rather than fresh wrapper allocation.
use super::brush_runtime::{BrushBlendBackend, BrushShip};
use crate::{
    framebuffer_target::FramebufferTarget, gl_data_holder::SourceGlDataHolder,
    source_ship::SourceShip,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

#[derive(Clone, Copy)]
pub(crate) enum BrushTarget {
    MaskStruts,
    Water,
}

pub(crate) struct SourceBrushShip {
    pub ship: Rc<RefCell<SourceShip>>,
    pub target: BrushTarget,
}
pub(crate) fn current_ship_provider(
    target: BrushTarget,
    mut current_ship: impl FnMut() -> Rc<RefCell<SourceShip>> + 'static,
) -> impl FnMut() -> Arc<dyn BrushShip> {
    let mut cached: Option<Arc<SourceBrushShip>> = None;
    move || {
        let ship = current_ship();
        if cached
            .as_ref()
            .is_none_or(|old| !Rc::ptr_eq(&old.ship, &ship))
        {
            cached = Some(Arc::new(SourceBrushShip { ship, target }));
        }
        cached.as_ref().unwrap().clone()
    }
}
pub(crate) struct SourceBrushBlend(pub Rc<RefCell<dyn crate::source_ship::ShipSceneStateBackend>>);
impl BrushBlendBackend for SourceBrushBlend {
    fn blending(&mut self, enabled: bool) {
        if enabled {
            self.0.borrow_mut().enable(3042);
        } else {
            self.0.borrow_mut().disable(3042);
        }
    }
}
impl BrushShip for SourceBrushShip {
    fn target(&self) -> Arc<dyn FramebufferTarget> {
        let ship = self.ship.borrow();
        let physics = ship.physics.borrow();
        match self.target {
            BrushTarget::MaskStruts => physics.mask_struts.source_texture(),
            BrushTarget::Water => physics.water.source_texture(),
        }
    }
    fn physics_filter(&self) -> Arc<dyn FramebufferTarget> {
        self.ship.borrow().physics.borrow().physics_filter.clone()
    }
    fn width(&self) -> i32 {
        self.ship.borrow().dat.width
    }
    fn height(&self) -> i32 {
        self.ship.borrow().dat.height
    }
    fn positions(&self) -> Arc<dyn FramebufferTarget> {
        self.ship.borrow().physics.borrow().pos_vel.source_texture()
    }
}
