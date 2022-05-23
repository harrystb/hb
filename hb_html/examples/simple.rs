use hb_html::objects::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw = reqwest::blocking::get("https://github.com/harrystb/hb")?.text()?;
    let doc = raw
        .replace("--></option></form><form", "--><form")
        .parse::<HtmlDocument>()?;
    assert_eq!(
        doc.find("a[title=hb_html]")
            .nodes()
            .iter()
            .map(|n| n.text())
            .next()
            .unwrap(),
        "hb_html"
    );
    Ok(())
}
