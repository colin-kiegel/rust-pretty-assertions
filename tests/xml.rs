use pretty_assertions::assert_eq;

// Test that a medium sized XML file does not run into a memory leak.
#[should_panic]
#[test]
fn assert_eq_xml() {
  let xml = include_str!("test.xml");
  let xml2 = xml.to_owned().replace("<job>", "\n<job>");

  assert_eq!(xml, xml2);
}
