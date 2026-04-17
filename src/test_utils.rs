use slack_morphism::prelude::*;

pub(crate) fn rich_text_block(value: serde_json::Value) -> SlackBlock {
    let mut value = value;
    if let Some(obj) = value.as_object_mut() {
        obj.remove("type");
        obj.entry("elements".to_string())
            .or_insert_with(|| serde_json::json!([]));
    }
    SlackBlock::RichText(serde_json::from_value(value).unwrap())
}
