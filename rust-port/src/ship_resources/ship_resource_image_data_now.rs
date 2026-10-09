//! Recovered ShipResource$imageDataNow$1.class continuation.
#[derive(Debug, PartialEq)]
pub(crate) enum Outcome<T> {
    Suspended,
    Complete(Option<T>),
}

pub(crate) struct AwaitImage<S> {
    pub label: i32,
    scope: Option<S>,
    saved_scope: Option<S>,
}
impl<S: Clone> AwaitImage<S> {
    pub fn new(scope: Option<S>) -> Self {
        Self {
            label: 0,
            scope,
            saved_scope: None,
        }
    }
    pub fn invoke_suspend<T>(
        &mut self,
        incoming: Result<Option<T>, String>,
        await_loader: impl FnOnce() -> Result<Outcome<T>, String>,
    ) -> Result<Outcome<T>, String> {
        match self.label {
            0 => {
                incoming?;
                self.saved_scope = self.scope.clone();
                self.label = 1;
                await_loader()
            }
            1 => {
                let _scope = self.saved_scope.clone();
                incoming.map(Outcome::Complete)
            }
            _ => Err("call to 'resume' before 'invoke' with coroutine".into()),
        }
    }
}

pub(crate) fn blocking<T>(
    await_loader: impl FnOnce() -> Result<Option<T>, String>,
) -> Result<Option<T>, String> {
    let mut callback = AwaitImage::new(Some(()));
    match callback.invoke_suspend(Ok(None), || await_loader().map(Outcome::Complete))? {
        Outcome::Complete(image) => Ok(image),
        Outcome::Suspended => unreachable!("blocking loader cannot return suspension"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    #[test]
    fn suspended_resume_retains_scope_and_exact_nullable_result() {
        let scope = Rc::new(());
        let mut callback = AwaitImage::new(Some(scope.clone()));
        assert_eq!(
            callback
                .invoke_suspend::<Rc<()>>(Ok(None), || Ok(Outcome::Suspended))
                .unwrap(),
            Outcome::Suspended
        );
        assert_eq!(callback.label, 1);
        assert!(Rc::ptr_eq(callback.saved_scope.as_ref().unwrap(), &scope));
        let image = Rc::new(());
        let Outcome::Complete(Some(actual)) = callback
            .invoke_suspend(Ok(Some(image.clone())), || panic!("await must not repeat"))
            .unwrap()
        else {
            panic!()
        };
        assert!(Rc::ptr_eq(&image, &actual));
        assert_eq!(
            callback
                .invoke_suspend::<()>(Ok(None), || panic!())
                .unwrap(),
            Outcome::Complete(None)
        );
    }
    #[test]
    fn failures_preserve_original_label_boundaries() {
        let mut callback = AwaitImage::<()>::new(None);
        assert!(
            callback
                .invoke_suspend::<()>(Err("incoming".into()), || panic!())
                .is_err()
        );
        assert_eq!(callback.label, 0);
        assert!(
            callback
                .invoke_suspend::<()>(Ok(None), || Err("await".into()))
                .is_err()
        );
        assert_eq!(callback.label, 1);
        assert_eq!(
            callback
                .invoke_suspend::<()>(Err("resume".into()), || panic!())
                .unwrap_err(),
            "resume"
        );
        callback.label = 2;
        assert!(
            callback
                .invoke_suspend::<()>(Ok(None), || panic!())
                .unwrap_err()
                .contains("resume")
        );
        assert_eq!(blocking::<()>(|| Ok(None)).unwrap(), None);
        assert_eq!(
            blocking::<()>(|| Err("loader".into())).unwrap_err(),
            "loader"
        );
    }
}
