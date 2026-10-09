//! Recovered ShipResource$imageData$1.class: non-suspending async body.
pub(crate) struct ImageLoader<S, G> {
    pub label: i32,
    scope: Option<S>,
    generate: G,
}
impl<S, G> ImageLoader<S, G> {
    pub fn new(scope: Option<S>, generate: G) -> Self {
        Self {
            label: 0,
            scope,
            generate,
        }
    }
    pub fn invoke_suspend<T>(&self, incoming: Result<(), String>) -> Result<Option<T>, String>
    where
        G: Fn() -> Result<Option<T>, String>,
    {
        if self.label != 0 {
            return Err("call to 'resume' before 'invoke' with coroutine".into());
        }
        incoming?;
        let _scope = &self.scope;
        (self.generate)()
    }
    pub fn create(&self, scope: Option<S>) -> Self
    where
        G: Clone,
    {
        Self::new(scope, self.generate.clone())
    }
    pub fn invoke<T>(&self, scope: Option<S>) -> Result<Option<T>, String>
    where
        G: Clone + Fn() -> Result<Option<T>, String>,
    {
        self.create(scope).invoke_suspend(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};
    #[test]
    fn body_preserves_nullable_result_identity_and_repeated_zero_label_calls() {
        let calls = Cell::new(0);
        let image = Rc::new(());
        let generate = || {
            calls.set(calls.get() + 1);
            Ok(Some(image.clone()))
        };
        let mut loader = ImageLoader::new(Some(Rc::new(())), generate);
        assert!(Rc::ptr_eq(
            &image,
            &loader.invoke_suspend(Ok(())).unwrap().unwrap()
        ));
        assert!(Rc::ptr_eq(
            &image,
            &loader.invoke_suspend(Ok(())).unwrap().unwrap()
        ));
        assert_eq!(calls.get(), 2);
        assert_eq!(loader.label, 0);
        assert_eq!(
            loader.invoke_suspend(Err("incoming".into())).unwrap_err(),
            "incoming"
        );
        assert_eq!(calls.get(), 2);
        loader.label = 1;
        assert!(
            loader
                .invoke_suspend(Err("incoming".into()))
                .unwrap_err()
                .contains("resume")
        );
        let scope = Rc::new(());
        let fresh = loader.create(Some(scope.clone()));
        assert_eq!(fresh.label, 0);
        assert!(Rc::ptr_eq(fresh.scope.as_ref().unwrap(), &scope));
        assert!(Rc::ptr_eq(&image, &loader.invoke(None).unwrap().unwrap()));
    }
    #[test]
    fn null_and_generator_failure_return_unchanged() {
        let empty = ImageLoader::new(None::<()>, || Ok(None::<()>));
        assert_eq!(empty.invoke_suspend(Ok(())).unwrap(), None);
        let failed = ImageLoader::new(None::<()>, || Err::<Option<()>, _>("decode".into()));
        assert_eq!(failed.invoke_suspend(Ok(())).unwrap_err(), "decode");
    }
}
