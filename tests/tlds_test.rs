use dnsimple_rust::dnsimple::{Paginate, Sort};
use crate::common::setup_mock_for;
mod common;

#[test]
fn test_list_tlds() {
    let setup = setup_mock_for("/tlds", "listTlds/success", "GET");
    let client = setup.0;

    let response = client.tlds().list_tlds(Sort { sort_by: "".to_string() }, Paginate { per_page: 30, page: 1 }).unwrap();
    let tlds = response.data.unwrap();

    assert_eq!(2, tlds.len());

    let tld = tlds.first().unwrap();

    assert_eq!("ac", tld.tld);
    assert_eq!(2, tld.tld_type);
    assert_eq!(false, tld.whois_privacy);
    assert_eq!(true, tld.auto_renew_only);
    assert_eq!(false, tld.idn);
    assert_eq!(1, tld.minimum_registration);
    assert_eq!(true, tld.registration_enabled);
    assert_eq!(true, tld.renewal_enabled);
    assert_eq!(false, tld.transfer_enabled);
    assert_eq!("ds", tld.dnssec_interface_type);
}

#[test]
fn test_get_tld() {
    let setup = setup_mock_for("/tlds/com", "getTld/success", "GET");
    let client = setup.0;
    let tld = String::from("com");

    let tld = client.tlds().get_tld(tld).unwrap().data.unwrap();

    assert_eq!("com", tld.tld);
    assert_eq!(1, tld.tld_type);
    assert_eq!(true, tld.whois_privacy);
    assert_eq!(false, tld.auto_renew_only);
    assert_eq!(true, tld.idn);
    assert_eq!(1, tld.minimum_registration);
    assert_eq!(true, tld.registration_enabled);
    assert_eq!(true, tld.renewal_enabled);
    assert_eq!(true, tld.transfer_enabled);
    assert_eq!("ds", tld.dnssec_interface_type);
}

#[test]
fn test_get_tld_extended_attributes() {
    let setup = setup_mock_for("/tlds/com/extended_attributes", "getTldExtendedAttributes/success", "GET");
    let client = setup.0;
    let tld = String::from("com");

    let response = client.tlds().get_tld_extended_attributes(tld).unwrap();
    let extended_attributes = response.data.unwrap();

    assert_eq!(4, extended_attributes.len());

    let extended_attribute = extended_attributes.first().unwrap();
    assert_eq!("uk_legal_type", extended_attribute.name);
    assert_eq!("Legal type of registrant contact", extended_attribute.description);
    assert_eq!(false, extended_attribute.required);

    let options = &extended_attribute.options;

    assert_eq!(17, options.len());

    let option = options.first().unwrap();
    assert_eq!("UK Individual", option.title);
    assert_eq!("IND", option.value);
    assert_eq!("UK Individual (our default value)", option.description);

}