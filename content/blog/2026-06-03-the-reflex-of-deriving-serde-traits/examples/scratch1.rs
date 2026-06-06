use crate::core::A;
use serde_json::json;

mod core {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde_with::{DeserializeAs, DisplayFromStr, SerializeAs, serde_as};

    #[serde_as]
    #[derive(Deserialize, Serialize)]
    pub struct A {
        #[serde_as(as = "DisplayFromStr")]
        pub mime: mime::Mime,
        #[serde_as(as = "DisplayFromStr")]
        pub number: u32,
    }
}

fn main() {
    let v: A = serde_json::from_value(json!({
        "mime": "text/plain",
        "number": "159",
    }))
    .unwrap();
    assert_eq!(mime::TEXT_PLAIN, v.mime);
    assert_eq!(159, v.number);

    let x = A {
        mime: mime::STAR_STAR,
        number: 777,
    };
    assert_eq!(
        json!({ "mime": "*/*", "number": "777" }),
        serde_json::to_value(x).unwrap()
    );
}
