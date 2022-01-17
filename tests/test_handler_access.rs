#[test]
fn test_handler() {
    use miette::{miette, Report};

    let error: Report = miette!("oh no!");
    let _ = error.handler();
}
