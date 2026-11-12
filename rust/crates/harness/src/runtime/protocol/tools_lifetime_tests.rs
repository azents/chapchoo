use super::*;
use chapchoo_crypto::BearerSecret;
use std::sync::atomic::AtomicBool;

struct Lease {
    closed: Arc<AtomicBool>,
    entered: Arc<Semaphore>,
    release: Arc<Semaphore>,
    finished: Arc<AtomicBool>,
}
impl crate::NativeToolLifetime for Lease {
    fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }
    fn verify(&self) -> crate::NativeToolsFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
    fn finish(&mut self) -> crate::NativeToolsFuture<'_, ()> {
        Box::pin(async {
            assert!(self.closed.load(Ordering::Acquire));
            self.entered.add_permits(1);
            self.release
                .acquire()
                .await
                .map_err(|error| crate::NativeToolsError::new(error.into()))?
                .forget();
            self.finished.store(true, Ordering::Release);
            Ok(())
        })
    }
}
#[tokio::test]
async fn cancelled_native_retirement_keeps_pending_tools_and_prior_cleanup_failure()
-> Result<(), TestError> {
    let saved = tempfile::tempdir()?;
    let mut owner = state(NativeStorage::open(saved.path())?)?;
    let closed = Arc::new(AtomicBool::new(false));
    let entered = Arc::new(Semaphore::new(0));
    let release = Arc::new(Semaphore::new(0));
    let finished = Arc::new(AtomicBool::new(false));
    owner.pending_tools = Some(crate::NativeToolProvision::new(
        crate::NativeMcpDeclaration::http("http://127.0.0.1:12345/mcp", BearerSecret::generate()?)?,
        Box::new(Lease {
            closed: closed.clone(),
            entered: entered.clone(),
            release: release.clone(),
            finished: finished.clone(),
        }),
    ));
    owner.credential_write = Some(tokio::spawn(async { Err(Error::Storage) }));
    {
        let retirement = owner.retire();
        tokio::pin!(retirement);
        tokio::select! {
            result = &mut retirement => return Err(format!("Retirement did not retain tool cleanup: {result:?}").into()),
            permit = entered.acquire() => permit?.forget(),
        }
    }
    assert!(closed.load(Ordering::Acquire));
    assert!(!finished.load(Ordering::Acquire));
    assert!(owner.pending_tools.is_some());
    release.add_permits(1);
    assert!(matches!(owner.retire().await, Err(Error::Storage)));
    assert!(finished.load(Ordering::Acquire));
    assert!(owner.pending_tools.is_none());
    Ok(())
}
