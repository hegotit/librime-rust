mod rime;

fn main() {
    let p1 = rime::common::PathExt::from("some/path");
    let p2 = rime::common::PathExt::from("to/somewhere");
    let p3 = p1.clone() / p2.clone();

    println!("{:?}", p3);

    let p4 = rime::common::PathExt::from("another/path");
    let p5 = p4.clone() / "to/another/place";

    println!("{:?}", p5);

    let mut p6 = rime::common::PathExt::from("base/path");
    p6 /= "extended/path";

    println!("{:?}", p6);

    #[cfg(feature = "logging")]
    {
        println!("{}", p3);
        println!("{}", p5);
        println!("{}", p6);
    }
}
