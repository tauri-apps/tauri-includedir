use std::env;

include!(concat!(env!("OUT_DIR"), "/data.rs"));

fn main() {
    println!("{:?}", FILES.get("data/foo"));

    for name in FILES.file_names() {
        println!("Found: {}", name);
    }
}

#[test]
fn test() {
    assert_eq!(
        FILES
            .get("data/foo")
            .expect("data/foo not present")
            .into_owned(),
        &[102, 111, 111, 13, 10]
    );

    let mut files = FILES.file_names().collect::<Vec<_>>();
    files.sort();
    assert_eq!(files, vec!["data/empty", "data/foo", "data/inner/boom"]);

    assert_eq!(
        FILES.get("data/inner/boom").unwrap().into_owned(),
        &[98, 111, 111, 109, 13, 10]
    );
    assert_eq!(
        FILES.get("data/foo").unwrap().into_owned(),
        &[102, 111, 111, 13, 10]
    );
    assert_eq!(FILES.get("data/empty").unwrap().into_owned(), &[]);
}
