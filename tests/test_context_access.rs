#[test]
fn test_context() {
    use miette::{miette, Report};

    let error: Report = miette!("oh no!");
    let _ = error.context();
}
