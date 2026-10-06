//! GameParameterProviderKt retains and replaces the source provider object.
//! Confined to the source engine thread; JVM cross-thread visibility remains pending.
use crate::game_parameters::SourceGameParameterProvider;
use std::{cell::RefCell, rc::Rc};
thread_local! {
    static PROVIDER: RefCell<Rc<RefCell<SourceGameParameterProvider>>> =
        RefCell::new(Rc::new(RefCell::new(SourceGameParameterProvider::default())));
}
pub fn get_game_parameter_provider() -> Rc<RefCell<SourceGameParameterProvider>> {
    PROVIDER.with(|provider| provider.borrow().clone())
}
pub fn set_game_parameter_provider(provider: Rc<RefCell<SourceGameParameterProvider>>) {
    PROVIDER.with(|current| *current.borrow_mut() = provider);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn getter_setter_identity_and_surviving_replaced_references() {
        let original = get_game_parameter_provider();
        assert!(Rc::ptr_eq(&original, &get_game_parameter_provider()));
        original.borrow_mut().set_tool(2.0);
        assert_eq!(get_game_parameter_provider().borrow().tool(), 2.0);
        let replacement = Rc::new(RefCell::new(SourceGameParameterProvider::default()));
        set_game_parameter_provider(replacement.clone());
        assert!(Rc::ptr_eq(&replacement, &get_game_parameter_provider()));
        assert_eq!(original.borrow().tool(), 2.0);
        set_game_parameter_provider(original);
    }
}
