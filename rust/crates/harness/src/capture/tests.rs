use super::*;
type TestError = Box<dyn std::error::Error + Send + Sync>;
#[test]
fn a_retained_source_has_one_writer_and_clones_share_that_owner() -> Result<(), TestError> {
    let (_dir, storage, log) = setup()?;
    let clone = log.clone();
    assert!(matches!(
        CaptureLog::open(&storage),
        Err(Error::CaptureUnavailable)
    ));
    drop(log);
    clone.append(&event("still-owned"), None)?;
    assert!(matches!(
        CaptureLog::open(&storage),
        Err(Error::CaptureUnavailable)
    ));
    drop(clone);
    let restored = CaptureLog::open(&storage)?;
    assert_eq!(
        restored
            .begin(&restored.snapshot()?, None)?
            .next_event()?
            .ok_or("Event")?
            .raw,
        event("still-owned")
    );
    Ok(())
}
fn event(text: &str) -> String {
    serde_json::json!({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"native","update":{"sessionUpdate":"_fixture","text":text}}}).to_string()
}
fn setup() -> Result<(tempfile::TempDir, NativeStorage, CaptureLog), TestError> {
    let dir = tempfile::tempdir()?;
    let storage = NativeStorage::open(dir.path())?;
    let log = CaptureLog::create(&storage)?;
    log.bind_native(&NativeConversationId::parse("native".into())?)?;
    Ok((dir, storage, log))
}
#[test]
fn finite_complete_boundary_does_not_follow_later_append() -> Result<(), TestError> {
    let (_dir, _storage, log) = setup()?;
    let first = log.append(
        &event("first"),
        Some(&EventOccurrenceId::parse("z".into())?),
    )?;
    let second = log.append(
        &event("second"),
        Some(&EventOccurrenceId::parse("a".into())?),
    )?;
    let snapshot = log.snapshot()?;
    let mut pass = log.begin(&snapshot, None)?;
    let later = log.append(&event("later"), None)?;
    assert_eq!(pass.boundary.through, Some(second.clone()));
    assert_eq!(pass.next_event()?.ok_or("First")?.position, first);
    assert_eq!(pass.next_event()?.ok_or("Second")?.position, second);
    assert!(pass.next_event()?.is_none());
    let mut catch_up = log.begin(&snapshot, Some(&second))?;
    assert_eq!(catch_up.boundary.through, Some(later.clone()));
    assert_eq!(catch_up.next_event()?.ok_or("Later")?.position, later);
    assert!(catch_up.next_event()?.is_none());
    Ok(())
}
#[test]
fn source_cursor_is_resolved_by_exact_occurrence_not_id_sorting_or_ordinal_alone()
-> Result<(), TestError> {
    let (_dir, _storage, log) = setup()?;
    let first = log.append(
        &event("first"),
        Some(&EventOccurrenceId::parse("z".into())?),
    )?;
    let second = log.append(
        &event("second"),
        Some(&EventOccurrenceId::parse("a".into())?),
    )?;
    let wrong_ordinal = SourcePosition::new(2, first.occurrence().clone())?;
    assert!(matches!(
        log.begin(&log.snapshot()?, Some(&wrong_ordinal)),
        Err(Error::CaptureConflict)
    ));
    let missing = SourcePosition::new(1, EventOccurrenceId::parse("missing".into())?)?;
    assert!(matches!(
        log.begin(&log.snapshot()?, Some(&missing)),
        Err(Error::CaptureUnavailable)
    ));
    assert!(matches!(
        log.begin(&SnapshotId::parse("another".into())?, None),
        Err(Error::CaptureUnavailable)
    ));
    let mut pass = log.begin(&log.snapshot()?, Some(&first))?;
    assert_eq!(pass.next_event()?.ok_or("Second")?.position, second);
    Ok(())
}
#[test]
fn supplied_occurrence_deduplicates_but_identical_uncorrelated_events_are_distinct()
-> Result<(), TestError> {
    let (_dir, _storage, log) = setup()?;
    let occurrence = EventOccurrenceId::parse("actual-event".into())?;
    let raw = event("same");
    let first = log.append(&raw, Some(&occurrence))?;
    assert_eq!(log.append(&raw, Some(&occurrence))?, first);
    assert!(matches!(
        log.append(&event("different"), Some(&occurrence)),
        Err(Error::CaptureConflict)
    ));
    let second = log.append(&raw, None)?;
    let third = log.append(&raw, None)?;
    assert_ne!(second.occurrence(), third.occurrence());
    assert_eq!(second.ordinal(), 2);
    assert_eq!(third.ordinal(), 3);
    let mut pass = log.begin(&log.snapshot()?, None)?;
    for position in [first, second, third] {
        assert_eq!(pass.next_event()?.ok_or("Event")?.position, position);
    }
    assert!(pass.next_event()?.is_none());
    Ok(())
}
#[test]
fn open_retains_raw_unknown_fields_full_utf8_body_and_binding_without_native_replay()
-> Result<(), TestError> {
    let (dir, storage, log) = setup()?;
    let raw = format!(
        " {{\"jsonrpc\":\"2.0\",\"method\":\"session/update\",\"params\":{{\"sessionId\":\"native\",\"update\":{{\"sessionUpdate\":\"_future\",\"unknown\":\"{}\"}}}}}} ",
        "한글".repeat(100_000)
    );
    let position = log.append(&raw, None)?;
    let snapshot = log.snapshot()?;
    drop(log);
    let reopened = CaptureLog::open(&storage)?;
    assert_eq!(reopened.snapshot()?, snapshot);
    assert_eq!(
        reopened.native_session()?.ok_or("Native binding")?.as_str(),
        "native"
    );
    let mut pass = reopened.begin(&snapshot, None)?;
    let retained = pass.next_event()?.ok_or("Retained event")?;
    assert_eq!(retained.raw, raw);
    assert_eq!(retained.position, position);
    assert!(pass.next_event()?.is_none());
    assert!(matches!(
        CaptureLog::create(&storage),
        Err(Error::CaptureUnavailable)
    ));
    assert!(dir.path().is_dir());
    Ok(())
}
#[test]
fn missing_or_partial_source_does_not_create_repair_or_truncate_retained_data()
-> Result<(), TestError> {
    let dir = tempfile::tempdir()?;
    let storage = NativeStorage::open(dir.path())?;
    assert!(matches!(
        CaptureLog::open(&storage),
        Err(Error::CaptureUnavailable)
    ));
    assert!(!storage.capture_root().join("current").exists());
    let log = CaptureLog::create(&storage)?;
    log.bind_native(&NativeConversationId::parse("native".into())?)?;
    log.append(&event("retained"), None)?;
    let source = log.0.directory.join("source.jsonl");
    drop(log);
    OpenOptions::new()
        .append(true)
        .open(&source)?
        .write_all(b"{\"ordinal\":2")?;
    let original = fs::read(&source)?;
    assert!(matches!(
        CaptureLog::open(&storage),
        Err(Error::CaptureCorrupt)
    ));
    assert!(matches!(
        CaptureLog::create(&storage),
        Err(Error::CaptureUnavailable)
    ));
    assert_eq!(fs::read(source)?, original);
    Ok(())
}
#[test]
fn binding_checks_events_received_before_setup_result_and_rejects_other_sessions()
-> Result<(), TestError> {
    let dir = tempfile::tempdir()?;
    let storage = NativeStorage::open(dir.path())?;
    let log = CaptureLog::create(&storage)?;
    log.append(&event("early"), None)?;
    assert!(log.native_session()?.is_none());
    assert!(matches!(
        log.bind_native(&NativeConversationId::parse("another".into())?),
        Err(Error::CaptureConflict)
    ));
    log.bind_native(&NativeConversationId::parse("native".into())?)?;
    let another = r#"{"method":"session/update","params":{"sessionId":"another","update":{}}}"#;
    assert!(matches!(
        log.append(another, None),
        Err(Error::CaptureConflict)
    ));
    assert!(matches!(
        log.append(
            r#"{"method":"auth/login","params":{"accessToken":"secret-marker"}}"#,
            None
        ),
        Err(Error::CaptureCorrupt)
    ));
    let mut pass = log.begin(&log.snapshot()?, None)?;
    assert_eq!(pass.next_event()?.ok_or("Event")?.raw, event("early"));
    assert!(pass.next_event()?.is_none());
    Ok(())
}
#[test]
fn finite_read_capacity_is_owned_through_pass_lifetime() -> Result<(), TestError> {
    let (_dir, _storage, log) = setup()?;
    let first = log.begin(&log.snapshot()?, None)?;
    let second = log.begin(&log.snapshot()?, None)?;
    assert!(matches!(
        log.begin(&log.snapshot()?, None),
        Err(Error::NotAdmitted)
    ));
    drop(first);
    let third = log.begin(&log.snapshot()?, None)?;
    drop((second, third));
    assert!(log.begin(&log.snapshot()?, None)?.next_event()?.is_none());
    Ok(())
}
#[test]
fn source_file_symlinks_are_rejected_without_reading_or_writing_foreign_data()
-> Result<(), TestError> {
    let (_dir, storage, log) = setup()?;
    let source = log.0.directory.join("source.jsonl");
    drop(log);
    let outside = tempfile::NamedTempFile::new()?;
    fs::write(outside.path(), b"secret-marker")?;
    fs::remove_file(&source)?;
    std::os::unix::fs::symlink(outside.path(), source)?;
    assert!(CaptureLog::open(&storage).is_err());
    assert_eq!(fs::read(outside.path())?, b"secret-marker");
    Ok(())
}
