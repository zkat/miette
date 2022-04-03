mod drop;

use self::drop::{DetectDrop, Flag};
use miette::{Diagnostic, Report, Result};

#[test]
fn test_convert() {
    let has_dropped = Flag::new();
    let error: Report = Report::new(DetectDrop::new(&has_dropped));
    let box_dyn = Box::<dyn Diagnostic + Send + Sync>::from(error);
    assert_eq!("oh no!", box_dyn.to_string());
    drop(box_dyn);
    assert!(has_dropped.get());
}

#[test]
fn test_question_mark() -> Result<(), Box<dyn Diagnostic>> {
    fn f() -> Result<()> {
        Ok(())
    }
    f()?;
    Ok(())
}

#[test]
fn test_convert_stderr() {
    let has_dropped = Flag::new();
    let error: Report = Report::new(DetectDrop::new(&has_dropped));
    let box_dyn = Box::<dyn std::error::Error + Send + Sync>::from(error);
    assert_eq!("oh no!", box_dyn.to_string());
    drop(box_dyn);
    assert!(has_dropped.get());
}

#[test]
fn test_question_mark_stderr() -> Result<(), Box<dyn std::error::Error>> {
    fn f() -> Result<()> {
        Ok(())
    }
    f()?;
    Ok(())
}
