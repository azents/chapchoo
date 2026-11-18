//! Fence the synchronous native publication against withdrawal of its route.
use super::*;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ContinuationRoute(Arc<Mutex<bool>>);
#[derive(Debug)]
pub struct ContinuationRouteLease(ContinuationRoute);

impl ContinuationRouteLease {
    pub fn open() -> Self {
        Self(ContinuationRoute(Arc::new(Mutex::new(true))))
    }
    pub fn route(&self) -> ContinuationRoute {
        self.0.clone()
    }
}
impl Drop for ContinuationRouteLease {
    fn drop(&mut self) {
        if let Ok(mut open) = self.0.0.lock() {
            *open = false;
        }
    }
}
impl ContinuationRoute {
    pub fn is_open(&self) -> bool {
        self.0.lock().is_ok_and(|open| *open)
    }
    pub(super) fn publish<T>(
        &self,
        publish: impl FnOnce() -> Result<T, Error>,
    ) -> Result<T, Error> {
        let open = self.0.lock().map_err(|_| Error::Protocol)?;
        if !*open {
            return Err(Error::NotAdmitted);
        }
        publish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacing_route_does_not_reopen_an_old_publication() -> Result<(), Error> {
        let first = ContinuationRouteLease::open();
        let old = first.route();
        drop(first);
        let second = ContinuationRouteLease::open();
        let calls = std::cell::Cell::new(0);
        let publish = || {
            calls.set(calls.get() + 1);
            Ok(())
        };
        assert!(matches!(old.publish(publish), Err(Error::NotAdmitted)));
        second.route().publish(publish)?;
        assert_eq!(calls.get(), 1);
        Ok(())
    }
}
