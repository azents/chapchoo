mod support;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[test]
fn native_fixture_retains_exercise_and_schema_drop_failures() -> Result<(), Error> {
    let primary = std::io::Error::other("Native exercise failed");
    let cleanup = std::io::Error::other("Native schema cleanup failed");
    let failure = support::combine::<(), ()>(Err(primary.into()), Err(cleanup.into()))
        .err()
        .ok_or("Missing combined fixture failure")?;
    assert_eq!(
        failure
            .source()
            .ok_or("Missing primary source")?
            .to_string(),
        "Native exercise failed"
    );
    assert!(failure.to_string().contains("Native schema cleanup failed"));
    Ok(())
}
