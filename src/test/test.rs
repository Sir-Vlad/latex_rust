#[test]
fn main() {
    run()
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut res = reqwest::get("https://www.ctan.org/topic/class")?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());
    println!("Body:\n{}", body);

    Ok(())
}
