use slack_morphism::prelude::*;

use crate::{
    references::SlackReferences,
    visitor::{
        visit_slack_block_image_element, visit_slack_block_mark_down_text,
        visit_slack_block_plain_text, visit_slack_context_block, visit_slack_divider_block,
        visit_slack_header_block, visit_slack_image_block, visit_slack_section_block,
        SlackRichTextBlock, Visitor,
    },
};

/// TODO: document this function
///
pub fn render_blocks_as_markdown(
    blocks: Vec<SlackBlock>,
    slack_references: SlackReferences,
) -> String {
    let mut block_renderer = MardownRenderer::new(slack_references);
    for block in blocks {
        block_renderer.visit_slack_block(&block);
    }
    block_renderer.sub_texts.join("")
}

struct MardownRenderer {
    pub sub_texts: Vec<String>,
    pub slack_references: SlackReferences,
}

impl MardownRenderer {
    pub fn new(slack_references: SlackReferences) -> Self {
        MardownRenderer {
            sub_texts: vec![],
            slack_references,
        }
    }
}

impl Visitor for MardownRenderer {
    fn visit_slack_section_block(&mut self, slack_section_block: &SlackSectionBlock) {
        let mut section_renderer = MardownRenderer::new(self.slack_references.clone());
        visit_slack_section_block(&mut section_renderer, slack_section_block);
        self.sub_texts.push(section_renderer.sub_texts.join(""));
    }

    fn visit_slack_block_plain_text(&mut self, slack_block_plain_text: &SlackBlockPlainText) {
        self.sub_texts.push(slack_block_plain_text.text.clone());
        visit_slack_block_plain_text(self, slack_block_plain_text);
    }

    fn visit_slack_header_block(&mut self, slack_header_block: &SlackHeaderBlock) {
        let mut header_renderer = MardownRenderer::new(self.slack_references.clone());
        visit_slack_header_block(&mut header_renderer, slack_header_block);
        self.sub_texts
            .push(format!("## {}", header_renderer.sub_texts.join("")));
    }

    fn visit_slack_divider_block(&mut self, slack_divider_block: &SlackDividerBlock) {
        self.sub_texts.push("---\n".to_string());
        visit_slack_divider_block(self, slack_divider_block);
    }

    fn visit_slack_image_block(&mut self, slack_image_block: &SlackImageBlock) {
        self.sub_texts.push(format!(
            "![{}]({})",
            slack_image_block.alt_text, slack_image_block.image_url
        ));
        visit_slack_image_block(self, slack_image_block);
    }

    fn visit_slack_block_image_element(
        &mut self,
        slack_block_image_element: &SlackBlockImageElement,
    ) {
        self.sub_texts.push(format!(
            "![{}]({})",
            slack_block_image_element.alt_text, slack_block_image_element.image_url
        ));
        visit_slack_block_image_element(self, slack_block_image_element);
    }

    fn visit_slack_block_mark_down_text(
        &mut self,
        slack_block_mark_down_text: &SlackBlockMarkDownText,
    ) {
        self.sub_texts.push(slack_block_mark_down_text.text.clone());
        visit_slack_block_mark_down_text(self, slack_block_mark_down_text);
    }

    fn visit_slack_context_block(&mut self, slack_context_block: &SlackContextBlock) {
        let mut section_renderer = MardownRenderer::new(self.slack_references.clone());
        visit_slack_context_block(&mut section_renderer, slack_context_block);
        self.sub_texts.push(section_renderer.sub_texts.join(""));
    }

    fn visit_slack_rich_text_block(&mut self, slack_rich_text_block: &SlackRichTextBlock) {
        self.sub_texts.push(render_rich_text_block_as_markdown(
            slack_rich_text_block.json_value.clone(),
            &self.slack_references,
        ));
    }
}

fn render_rich_text_block_as_markdown(
    json_value: serde_json::Value,
    slack_references: &SlackReferences,
) -> String {
    match json_value.get("elements") {
        Some(serde_json::Value::Array(elements)) => elements
            .iter()
            .map(|element| {
                match (
                    element.get("type").map(|t| t.as_str()),
                    element.get("style"),
                    element.get("elements"),
                ) {
                    (
                        Some(Some("rich_text_section")),
                        None,
                        Some(serde_json::Value::Array(elements)),
                    ) => render_rich_text_section_elements(elements, slack_references),
                    (
                        Some(Some("rich_text_list")),
                        Some(serde_json::Value::String(style)),
                        Some(serde_json::Value::Array(elements)),
                    ) => render_rich_text_list_elements(elements, style, slack_references),
                    (
                        Some(Some("rich_text_preformatted")),
                        None,
                        Some(serde_json::Value::Array(elements)),
                    ) => render_rich_text_preformatted_elements(elements, slack_references),
                    (
                        Some(Some("rich_text_quote")),
                        None,
                        Some(serde_json::Value::Array(elements)),
                    ) => render_rich_text_quote_elements(elements, slack_references),
                    _ => "".to_string(),
                }
            })
            .collect::<Vec<String>>()
            .join(""),
        _ => "".to_string(),
    }
}

fn render_rich_text_section_elements(
    elements: &[serde_json::Value],
    slack_references: &SlackReferences,
) -> String {
    elements
        .iter()
        .map(|e| render_rich_text_section_element(e, slack_references))
        .collect::<Vec<String>>()
        .join("")
}

fn render_rich_text_list_elements(
    elements: &[serde_json::Value],
    style: &str,
    slack_references: &SlackReferences,
) -> String {
    let list_style = if style == "ordered" { "1." } else { "-" };
    elements
        .iter()
        .filter_map(|element| {
            if let Some(serde_json::Value::Array(elements)) = element.get("elements") {
                Some(render_rich_text_section_elements(
                    elements,
                    slack_references,
                ))
            } else {
                None
            }
        })
        .map(|element| format!("{list_style} {element}"))
        .collect::<Vec<String>>()
        .join("\n")
}

fn render_rich_text_preformatted_elements(
    elements: &[serde_json::Value],
    slack_references: &SlackReferences,
) -> String {
    format!(
        "```{}```",
        render_rich_text_section_elements(elements, slack_references)
    )
}

fn render_rich_text_quote_elements(
    elements: &[serde_json::Value],
    slack_references: &SlackReferences,
) -> String {
    format!(
        "> {}",
        render_rich_text_section_elements(elements, slack_references)
    )
}

fn render_rich_text_section_element(
    element: &serde_json::Value,
    slack_references: &SlackReferences,
) -> String {
    match element.get("type").map(|t| t.as_str()) {
        Some(Some("text")) => {
            let Some(serde_json::Value::String(text)) = element.get("text") else {
                return "".to_string();
            };
            let style = element.get("style");
            let text = apply_bold_style(text.to_string(), style);
            let text = apply_italic_style(text, style);
            let text = apply_strike_style(text, style);
            apply_code_style(text, style)
        }
        Some(Some("channel")) => {
            let Some(serde_json::Value::String(channel_id)) = element.get("channel_id") else {
                return "".to_string();
            };
            let channel_rendered = if let Some(Some(channel_name)) = slack_references
                .channels
                .get(&SlackChannelId(channel_id.clone()))
            {
                channel_name
            } else {
                channel_id
            };
            let style = element.get("style");
            let channel_rendered = apply_bold_style(format!("#{channel_rendered}"), style);
            let channel_rendered = apply_italic_style(channel_rendered, style);
            let channel_rendered = apply_strike_style(channel_rendered, style);
            apply_code_style(channel_rendered, style)
        }
        Some(Some("user")) => {
            let Some(serde_json::Value::String(user_id)) = element.get("user_id") else {
                return "".to_string();
            };
            let user_rendered = if let Some(Some(user_name)) =
                slack_references.users.get(&SlackUserId(user_id.clone()))
            {
                user_name
            } else {
                user_id
            };
            let style = element.get("style");
            let user_rendered = apply_bold_style(format!("@{user_rendered}"), style);
            let user_rendered = apply_italic_style(user_rendered, style);
            let user_rendered = apply_strike_style(user_rendered, style);
            apply_code_style(user_rendered, style)
        }
        Some(Some("usergroup")) => {
            let Some(serde_json::Value::String(usergroup_id)) = element.get("usergroup_id") else {
                return "".to_string();
            };
            let usergroup_rendered = if let Some(Some(usergroup_name)) = slack_references
                .usergroups
                .get(&SlackUserGroupId(usergroup_id.clone()))
            {
                usergroup_name
            } else {
                usergroup_id
            };
            let style = element.get("style");
            let usergroup_rendered = apply_bold_style(format!("@{usergroup_rendered}"), style);
            let usergroup_rendered = apply_italic_style(usergroup_rendered, style);
            let usergroup_rendered = apply_strike_style(usergroup_rendered, style);
            apply_code_style(usergroup_rendered, style)
        }
        Some(Some("emoji")) => {
            let Some(serde_json::Value::String(name)) = element.get("name") else {
                return "".to_string();
            };
            let splitted = name.split("::skin-tone-").collect::<Vec<&str>>();
            let Some(first) = splitted.first() else {
                return format!(":{}:", name);
            };
            let Some(emoji) = emojis::get_by_shortcode(first) else {
                return format!(":{}:", name);
            };
            let Some(skin_tone) = splitted.get(1).and_then(|s| s.parse::<usize>().ok()) else {
                return emoji.to_string();
            };
            let Some(mut skin_tones) = emoji.skin_tones() else {
                return emoji.to_string();
            };
            let Some(skinned_emoji) = skin_tones.nth(skin_tone - 1) else {
                return emoji.to_string();
            };
            skinned_emoji.to_string()
        }
        Some(Some("link")) => {
            let (Some(serde_json::Value::String(url)), Some(serde_json::Value::String(text))) =
                (element.get("url"), element.get("text"))
            else {
                return "".to_string();
            };
            let style = element.get("style");
            let url = apply_bold_style(render_url_as_markdown(url, text), style);
            let url = apply_italic_style(url, style);
            let url = apply_strike_style(url, style);
            apply_code_style(url, style)
        }
        _ => "".to_string(),
    }
}

fn render_url_as_markdown(url: &str, text: &str) -> String {
    format!("[{}]({})", text, url)
}

fn apply_bold_style(text: String, style: Option<&serde_json::Value>) -> String {
    if is_styled(style, "bold") {
        format!("*{}*", text)
    } else {
        text
    }
}

fn apply_italic_style(text: String, style: Option<&serde_json::Value>) -> String {
    if is_styled(style, "italic") {
        format!("_{}_", text)
    } else {
        text
    }
}

fn apply_strike_style(text: String, style: Option<&serde_json::Value>) -> String {
    if is_styled(style, "strike") {
        format!("~{}~", text)
    } else {
        text
    }
}

fn apply_code_style(text: String, style: Option<&serde_json::Value>) -> String {
    if is_styled(style, "code") {
        format!("`{}`", text)
    } else {
        text
    }
}

fn is_styled(style: Option<&serde_json::Value>, style_name: &str) -> bool {
    style
        .and_then(|s| s.get(style_name).map(|b| b.as_bool()))
        .flatten()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use url::Url;

    use super::*;

    #[test]
    fn test_empty_input() {
        assert_eq!(
            render_blocks_as_markdown(vec![], SlackReferences::default()),
            "".to_string()
        );
    }

    #[test]
    fn test_with_image() {
        let blocks = vec![
            SlackBlock::Image(SlackImageBlock::new(
                Url::parse("https://example.com/image.png").unwrap(),
                "Image".to_string(),
            )),
            SlackBlock::Image(SlackImageBlock::new(
                Url::parse("https://example.com/image2.png").unwrap(),
                "Image2".to_string(),
            )),
        ];
        assert_eq!(
            render_blocks_as_markdown(blocks, SlackReferences::default()),
            "![Image](https://example.com/image.png)![Image2](https://example.com/image2.png)"
                .to_string()
        );
    }

    #[test]
    fn test_with_divider() {
        let blocks = vec![
            SlackBlock::Divider(SlackDividerBlock::new()),
            SlackBlock::Divider(SlackDividerBlock::new()),
        ];
        assert_eq!(
            render_blocks_as_markdown(blocks, SlackReferences::default()),
            "---\n---\n".to_string()
        );
    }

    #[test]
    fn test_with_input() {
        // No rendering
        let blocks = vec![SlackBlock::Input(SlackInputBlock::new(
            "label".into(),
            SlackInputBlockElement::PlainTextInput(SlackBlockPlainTextInputElement::new(
                "id".into(),
            )),
        ))];
        assert_eq!(
            render_blocks_as_markdown(blocks, SlackReferences::default()),
            "".to_string()
        );
    }

    #[test]
    fn test_with_action() {
        // No rendering
        let blocks = vec![SlackBlock::Actions(SlackActionsBlock::new(vec![]))];
        assert_eq!(
            render_blocks_as_markdown(blocks, SlackReferences::default()),
            "".to_string()
        );
    }

    #[test]
    fn test_with_file() {
        // No rendering
        let blocks = vec![SlackBlock::File(SlackFileBlock::new("external_id".into()))];
        assert_eq!(
            render_blocks_as_markdown(blocks, SlackReferences::default()),
            "".to_string()
        );
    }

    #[test]
    fn test_with_event() {
        // No rendering
        let blocks = vec![SlackBlock::Event(serde_json::json!({}))];
        assert_eq!(
            render_blocks_as_markdown(blocks, SlackReferences::default()),
            "".to_string()
        );
    }

    #[test]
    fn test_header() {
        let blocks = vec![SlackBlock::Header(SlackHeaderBlock::new("Text".into()))];
        assert_eq!(
            render_blocks_as_markdown(blocks, SlackReferences::default()),
            "## Text".to_string()
        );
    }

    mod section {
        use super::*;

        #[test]
        fn test_with_plain_text() {
            let blocks = vec![
                SlackBlock::Section(SlackSectionBlock::new().with_text(SlackBlockText::Plain(
                    SlackBlockPlainText::new("Text".to_string()),
                ))),
                SlackBlock::Section(SlackSectionBlock::new().with_text(SlackBlockText::Plain(
                    SlackBlockPlainText::new("Text2".to_string()),
                ))),
            ];
            assert_eq!(
                render_blocks_as_markdown(blocks, SlackReferences::default()),
                "TextText2".to_string()
            );
        }

        #[test]
        fn test_with_markdown() {
            let blocks = vec![
                SlackBlock::Section(SlackSectionBlock::new().with_text(SlackBlockText::MarkDown(
                    SlackBlockMarkDownText::new("Text".to_string()),
                ))),
                SlackBlock::Section(SlackSectionBlock::new().with_text(SlackBlockText::MarkDown(
                    SlackBlockMarkDownText::new("Text2".to_string()),
                ))),
            ];
            assert_eq!(
                render_blocks_as_markdown(blocks, SlackReferences::default()),
                "TextText2".to_string()
            );
        }

        #[test]
        fn test_with_fields() {
            let blocks = vec![
                SlackBlock::Section(SlackSectionBlock::new().with_fields(vec![
                    SlackBlockText::Plain(SlackBlockPlainText::new("Text11".to_string())),
                    SlackBlockText::Plain(SlackBlockPlainText::new("Text12".to_string())),
                ])),
                SlackBlock::Section(SlackSectionBlock::new().with_fields(vec![
                    SlackBlockText::Plain(SlackBlockPlainText::new("Text21".to_string())),
                    SlackBlockText::Plain(SlackBlockPlainText::new("Text22".to_string())),
                ])),
            ];
            assert_eq!(
                render_blocks_as_markdown(blocks, SlackReferences::default()),
                "Text11Text12Text21Text22".to_string()
            );
        }

        #[test]
        fn test_with_fields_and_text() {
            let blocks = vec![
                SlackBlock::Section(
                    SlackSectionBlock::new()
                        .with_text(SlackBlockText::MarkDown(SlackBlockMarkDownText::new(
                            "Text1".to_string(),
                        )))
                        .with_fields(vec![
                            SlackBlockText::Plain(SlackBlockPlainText::new("Text11".to_string())),
                            SlackBlockText::Plain(SlackBlockPlainText::new("Text12".to_string())),
                        ]),
                ),
                SlackBlock::Section(
                    SlackSectionBlock::new()
                        .with_text(SlackBlockText::MarkDown(SlackBlockMarkDownText::new(
                            "Text2".to_string(),
                        )))
                        .with_fields(vec![
                            SlackBlockText::Plain(SlackBlockPlainText::new("Text21".to_string())),
                            SlackBlockText::Plain(SlackBlockPlainText::new("Text22".to_string())),
                        ]),
                ),
            ];
            assert_eq!(
                render_blocks_as_markdown(blocks, SlackReferences::default()),
                "Text1Text11Text12Text2Text21Text22".to_string()
            );
        }
    }

    mod context {
        use super::*;

        #[test]
        fn test_with_image() {
            let blocks = vec![SlackBlock::Context(SlackContextBlock::new(vec![
                SlackContextBlockElement::Image(SlackBlockImageElement::new(
                    "https://example.com/image.png".to_string(),
                    "Image".to_string(),
                )),
                SlackContextBlockElement::Image(SlackBlockImageElement::new(
                    "https://example.com/image2.png".to_string(),
                    "Image2".to_string(),
                )),
            ]))];
            assert_eq!(
                render_blocks_as_markdown(blocks, SlackReferences::default()),
                "![Image](https://example.com/image.png)![Image2](https://example.com/image2.png)"
                    .to_string()
            );
        }

        #[test]
        fn test_with_plain_text() {
            let blocks = vec![SlackBlock::Context(SlackContextBlock::new(vec![
                SlackContextBlockElement::Plain(SlackBlockPlainText::new("Text".to_string())),
                SlackContextBlockElement::Plain(SlackBlockPlainText::new("Text2".to_string())),
            ]))];
            assert_eq!(
                render_blocks_as_markdown(blocks, SlackReferences::default()),
                "TextText2".to_string()
            );
        }

        #[test]
        fn test_with_markdown() {
            let blocks = vec![SlackBlock::Context(SlackContextBlock::new(vec![
                SlackContextBlockElement::MarkDown(SlackBlockMarkDownText::new("Text".to_string())),
                SlackContextBlockElement::MarkDown(SlackBlockMarkDownText::new(
                    "Text2".to_string(),
                )),
            ]))];
            assert_eq!(
                render_blocks_as_markdown(blocks, SlackReferences::default()),
                "TextText2".to_string()
            );
        }
    }

    mod rich_text {
        use super::*;

        #[test]
        fn test_with_empty_json() {
            let blocks = vec![
                SlackBlock::RichText(serde_json::json!({})),
                SlackBlock::RichText(serde_json::json!({})),
            ];
            assert_eq!(
                render_blocks_as_markdown(blocks, SlackReferences::default()),
                "".to_string()
            );
        }

        mod rich_text_section {
            use super::*;

            mod text_element {
                use super::*;

                #[test]
                fn test_with_text() {
                    let blocks = vec![
                        SlackBlock::RichText(serde_json::json!({
                            "type": "rich_text",
                            "elements": [
                                {
                                    "type": "rich_text_section",
                                    "elements": [
                                        {
                                            "type": "text",
                                            "text": "Text111"
                                        },
                                        {
                                            "type": "text",
                                            "text": "Text112"
                                        }
                                    ]
                                },
                                {
                                    "type": "rich_text_section",
                                    "elements": [
                                        {
                                            "type": "text",
                                            "text": "Text121"
                                        },
                                        {
                                            "type": "text",
                                            "text": "Text122"
                                        }
                                    ]
                                }
                            ]
                        })),
                        SlackBlock::RichText(serde_json::json!({
                            "type": "rich_text",
                            "elements": [
                                {
                                    "type": "rich_text_section",
                                    "elements": [
                                        {
                                            "type": "text",
                                            "text": "Text211"
                                        },
                                        {
                                            "type": "text",
                                            "text": "Text212"
                                        }
                                    ]
                                },
                                {
                                    "type": "rich_text_section",
                                    "elements": [
                                        {
                                            "type": "text",
                                            "text": "Text221"
                                        },
                                        {
                                            "type": "text",
                                            "text": "Text222"
                                        }
                                    ]
                                }
                            ]
                        })),
                    ];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        "Text111Text112Text121Text122Text211Text212Text221Text222".to_string()
                    );
                }

                #[test]
                fn test_with_bold_text() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Text",
                                        "style": {
                                            "bold": true
                                        }
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        "*Text*".to_string()
                    );
                }

                #[test]
                fn test_with_italic_text() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Text",
                                        "style": {
                                            "italic": true
                                        }
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        "_Text_".to_string()
                    );
                }

                #[test]
                fn test_with_strike_text() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Text",
                                        "style": {
                                            "strike": true
                                        }
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        "~Text~".to_string()
                    );
                }

                #[test]
                fn test_with_code_text() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Text",
                                        "style": {
                                            "code": true
                                        }
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        "`Text`".to_string()
                    );
                }

                #[test]
                fn test_with_styled_text() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Text",
                                        "style": {
                                            "bold": true,
                                            "italic": true,
                                            "strike": true
                                        }
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        "~_*Text*_~".to_string()
                    );
                }
            }

            mod channel_element {
                use super::*;

                #[test]
                fn test_with_channel_id() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "channel",
                                        "channel_id": "C0123456"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        "#C0123456".to_string()
                    );
                }

                #[test]
                fn test_with_channel_id_and_reference() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "channel",
                                        "channel_id": "C0123456"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(
                            blocks,
                            SlackReferences {
                                channels: HashMap::from([(
                                    SlackChannelId("C0123456".to_string()),
                                    Some("general".to_string())
                                )]),
                                ..SlackReferences::default()
                            }
                        ),
                        "#general".to_string()
                    );
                }
            }

            mod user_element {
                use super::*;

                #[test]
                fn test_with_user_id() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "user",
                                        "user_id": "user1"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        "@user1".to_string()
                    );
                }

                #[test]
                fn test_with_user_id_and_reference() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "user",
                                        "user_id": "user1"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(
                            blocks,
                            SlackReferences {
                                users: HashMap::from([(
                                    SlackUserId("user1".to_string()),
                                    Some("John Doe".to_string())
                                )]),
                                ..SlackReferences::default()
                            }
                        ),
                        "@John Doe".to_string()
                    );
                }
            }

            mod usergroup_element {
                use super::*;

                #[test]
                fn test_with_usergroup_id() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "usergroup",
                                        "usergroup_id": "group1"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        "@group1".to_string()
                    );
                }

                #[test]
                fn test_with_usergroup_id_and_reference() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "usergroup",
                                        "usergroup_id": "group1"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(
                            blocks,
                            SlackReferences {
                                usergroups: HashMap::from([(
                                    SlackUserGroupId("group1".to_string()),
                                    Some("Admins".to_string())
                                )]),
                                ..SlackReferences::default()
                            }
                        ),
                        "@Admins".to_string()
                    );
                }
            }

            mod link_element {
                use super::*;

                #[test]
                fn test_with_url() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "link",
                                        "text": "example",
                                        "url": "https://example.com"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        "[example](https://example.com)".to_string()
                    );
                }
            }

            mod emoji_element {
                use super::*;

                #[test]
                fn test_with_emoji() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "emoji",
                                        "name": "wave"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        "👋".to_string()
                    );
                }

                #[test]
                fn test_with_emoji_with_skin_tone() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "emoji",
                                        "name": "wave::skin-tone-2"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        "👋🏻".to_string()
                    );
                }

                #[test]
                fn test_with_emoji_with_unknown_skin_tone() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "emoji",
                                        "name": "wave::skin-tone-42"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        "👋".to_string()
                    );
                }

                #[test]
                fn test_with_unknown_emoji() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "emoji",
                                        "name": "bbb"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default()),
                        ":bbb:".to_string()
                    );
                }
            }
        }

        mod rich_text_list {
            use super::*;

            #[test]
            fn test_with_ordered_list() {
                let blocks = vec![SlackBlock::RichText(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_list",
                            "style": "ordered",
                            "elements": [
                                {
                                    "type": "rich_text_section",
                                    "elements": [
                                        {
                                            "type": "text",
                                            "text": "Text1"
                                        }
                                    ]
                                },
                                {
                                    "type": "rich_text_section",
                                    "elements": [
                                        {
                                            "type": "text",
                                            "text": "Text2"
                                        }
                                    ]
                                }
                            ]
                         },
                    ]
                }))];
                assert_eq!(
                    render_blocks_as_markdown(blocks, SlackReferences::default()),
                    "1. Text1\n1. Text2".to_string()
                );
            }

            #[test]
            fn test_with_bullet_list() {
                let blocks = vec![SlackBlock::RichText(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_list",
                            "style": "bullet",
                            "elements": [
                                {
                                    "type": "rich_text_section",
                                    "elements": [
                                        {
                                            "type": "text",
                                            "text": "Text1"
                                        }
                                    ]
                                },
                                {
                                    "type": "rich_text_section",
                                    "elements": [
                                        {
                                            "type": "text",
                                            "text": "Text2"
                                        }
                                    ]
                                }
                            ]
                        },
                    ]
                }))];
                assert_eq!(
                    render_blocks_as_markdown(blocks, SlackReferences::default()),
                    "- Text1\n- Text2".to_string()
                );
            }
        }

        mod rich_text_preformatted {
            use super::*;

            #[test]
            fn test_with_text() {
                let blocks = vec![SlackBlock::RichText(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_preformatted",
                            "elements": [
                                {
                                    "type": "text",
                                    "text": "Text1"
                                },
                                {
                                    "type": "text",
                                    "text": "Text2"
                                }
                            ]
                        },
                    ]
                }))];
                assert_eq!(
                    render_blocks_as_markdown(blocks, SlackReferences::default()),
                    "```Text1Text2```".to_string()
                );
            }
        }

        mod rich_text_quote {
            use super::*;

            #[test]
            fn test_with_text() {
                let blocks = vec![SlackBlock::RichText(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_quote",
                            "elements": [
                                {
                                    "type": "text",
                                    "text": "Text1"
                                },
                                {
                                    "type": "text",
                                    "text": "Text2"
                                }
                            ]
                        },
                    ]
                }))];
                assert_eq!(
                    render_blocks_as_markdown(blocks, SlackReferences::default()),
                    "> Text1Text2".to_string()
                );
            }
        }
    }
}
