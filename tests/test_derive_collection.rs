use std::{
    collections::{LinkedList, VecDeque},
    ops::Range,
};

// Testing of the `diagnostic` attr used by derive(Diagnostic)
use miette::{Diagnostic, LabeledSpan, MietteSourceCode, SourceSpan};
use thiserror::Error;

#[test]
fn attr_collection_in_enum() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    enum MyBad {
        Only {
            #[source_code]
            src: MietteSourceCode<String>,
            #[label("this bit here")]
            highlight: SourceSpan,
            #[label(collection, "and here")]
            highlight2: Vec<SourceSpan>,
        },
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad::Only {
        src: MietteSourceCode::new(src).with_name("bad_file.rs"),
        highlight: (9, 4).into(),
        highlight2: vec![(1, 2).into(), (3, 4).into()],
    };
    let mut label_iter = err.labels().unwrap();
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("this bit here".into()), 9usize, 4usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 1usize, 2usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 3usize, 4usize);
    assert_eq!(err_span, expectation);
}

#[test]
fn attr_collection_in_struct() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    struct MyBad {
        #[source_code]
        src: MietteSourceCode<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
        #[label(collection, "and here")]
        highlight2: Vec<SourceSpan>,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: MietteSourceCode::new(src).with_name("bad_file.rs"),
        highlight: (9, 4).into(),
        highlight2: vec![(1, 2).into(), (3, 4).into()],
    };
    let mut label_iter = err.labels().unwrap();
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("this bit here".into()), 9usize, 4usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 1usize, 2usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 3usize, 4usize);
    assert_eq!(err_span, expectation);
}

#[test]
fn attr_collection_as_deque() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    struct MyBad {
        #[source_code]
        src: MietteSourceCode<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
        #[label(collection, "and here")]
        highlight2: VecDeque<SourceSpan>,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: MietteSourceCode::new(src).with_name("bad_file.rs"),
        highlight: (9, 4).into(),
        highlight2: VecDeque::from([(1, 2).into(), (3, 4).into()]),
    };
    let mut label_iter = err.labels().unwrap();
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("this bit here".into()), 9usize, 4usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 1usize, 2usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 3usize, 4usize);
    assert_eq!(err_span, expectation);
}

#[test]
fn attr_collection_as_linked_list() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    struct MyBad {
        #[source_code]
        src: MietteSourceCode<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
        #[label(collection, "and here")]
        highlight2: LinkedList<SourceSpan>,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: MietteSourceCode::new(src).with_name("bad_file.rs"),
        highlight: (9, 4).into(),
        highlight2: LinkedList::from([(1, 2).into(), (3, 4).into()]),
    };
    let mut label_iter = err.labels().unwrap();
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("this bit here".into()), 9usize, 4usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 1usize, 2usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 3usize, 4usize);
    assert_eq!(err_span, expectation);
}

#[test]
fn attr_collection_of_tuple() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    struct MyBad {
        #[source_code]
        src: MietteSourceCode<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
        #[label(collection, "and here")]
        highlight2: Vec<(usize, usize)>,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: MietteSourceCode::new(src).with_name("bad_file.rs"),
        highlight: (9, 4).into(),
        highlight2: vec![(1, 2), (3, 4)],
    };
    let mut label_iter = err.labels().unwrap();
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("this bit here".into()), 9usize, 4usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 1usize, 2usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 3usize, 4usize);
    assert_eq!(err_span, expectation);
}

#[test]
fn attr_collection_of_range() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    struct MyBad {
        #[source_code]
        src: MietteSourceCode<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
        #[label(collection, "and here")]
        highlight2: Vec<Range<usize>>,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: MietteSourceCode::new(src).with_name("bad_file.rs"),
        highlight: (9, 4).into(),
        highlight2: vec![1..3, 3..7],
    };
    let mut label_iter = err.labels().unwrap();
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("this bit here".into()), 9usize, 4usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 1usize, 2usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 3usize, 4usize);
    assert_eq!(err_span, expectation);
}

#[test]
fn attr_collection_of_labeled_span_in_struct() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    struct MyBad {
        #[source_code]
        src: MietteSourceCode<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
        #[label(collection, "then there")]
        highlight2: Vec<LabeledSpan>,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: MietteSourceCode::new(src).with_name("bad_file.rs"),
        highlight: (9, 4).into(),
        highlight2: vec![
            LabeledSpan::new_with_span(Some("continuing here".to_string()), (1, 2)),
            LabeledSpan::new_with_span(None, (3, 4)),
        ],
    };
    let mut label_iter = err.labels().unwrap();
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("this bit here".into()), 9usize, 4usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("continuing here".into()), 1usize, 2usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("then there".into()), 3usize, 4usize);
    assert_eq!(err_span, expectation);
}

#[test]
fn attr_collection_of_labeled_span_in_enum() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    enum MyBad {
        Only {
            #[source_code]
            src: MietteSourceCode<String>,
            #[label("this bit here")]
            highlight: SourceSpan,
            #[label(collection, "then there")]
            highlight2: Vec<LabeledSpan>,
        },
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad::Only {
        src: MietteSourceCode::new(src).with_name("bad_file.rs"),
        highlight: (9, 4).into(),
        highlight2: vec![
            LabeledSpan::new_with_span(Some("continuing here".to_string()), (1, 2)),
            LabeledSpan::new_with_span(None, (3, 4)),
        ],
    };
    let mut label_iter = err.labels().unwrap();
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("this bit here".into()), 9usize, 4usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("continuing here".into()), 1usize, 2usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("then there".into()), 3usize, 4usize);
    assert_eq!(err_span, expectation);
}

#[test]
fn attr_collection_multi() {
    #[derive(Debug, Diagnostic, Error)]
    #[error("oops!")]
    struct MyBad {
        #[source_code]
        src: MietteSourceCode<String>,
        #[label("this bit here")]
        highlight: SourceSpan,
        #[label(collection, "and here")]
        highlight2: Vec<SourceSpan>,
        #[label(collection, "and there")]
        highlight3: Vec<SourceSpan>,
    }

    let src = "source\n  text\n    here".to_string();
    let err = MyBad {
        src: MietteSourceCode::new(src).with_name("bad_file.rs"),
        highlight: (9, 4).into(),
        highlight2: vec![(1, 2).into(), (3, 4).into()],
        highlight3: vec![(5, 6).into(), (7, 8).into()],
    };
    let mut label_iter = err.labels().unwrap();
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("this bit here".into()), 9usize, 4usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 1usize, 2usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and here".into()), 3usize, 4usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and there".into()), 5usize, 6usize);
    assert_eq!(err_span, expectation);
    let err_span = label_iter.next().unwrap();
    let expectation = LabeledSpan::new(Some("and there".into()), 7usize, 8usize);
    assert_eq!(err_span, expectation);
}
