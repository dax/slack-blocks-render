//! Slack blocks render is a Rust library to render [Slack blocks](https://api.slack.com/reference/block-kit) as Markdown.
//!
//! # Usage
//!
//! First, add the `slack_blocks_render` crate as a dependency:
//! ```sh
//! cargo add slack_blocks_render
//! ```
//!
//! Slack blocks render uses the `slack_morphism` data model as input so you should also add it as a dependency:
//! ```sh
//! cargo add slack_morphism
//! ```
//!
//! ```
//! use slack_morphism::prelude::*;
//! use slack_blocks_render::render_blocks_as_markdown;
//!
//! let blocks: Vec<SlackBlock> = vec![
//!     SlackBlock::RichText(serde_json::json!({
//!         "type": "rich_text",
//!         "elements": [
//!             {
//!                 "type": "rich_text_section",
//!                 "elements": [
//!                     {
//!                         "type": "text",
//!                         "text": "Hello World"
//!                     }
//!                 ]
//!             },
//!         ]
//!     })),
//! ];
//! let markdown_text = render_blocks_as_markdown(blocks);
//! ```
use slack_morphism::prelude::*;

/// TODO: document this function
///
pub fn render_blocks_as_markdown(blocks: Vec<SlackBlock>) -> String {
    blocks
        .into_iter()
        .map(|block| match block {
            SlackBlock::Section(section) => render_section_block_as_markdown(section),
            SlackBlock::Header(header) => render_header_block_as_markdown(header),
            SlackBlock::Divider(_) => "---".to_string(),
            SlackBlock::Image(image) => render_image_block_as_markdown(image),
            SlackBlock::Actions(actions) => render_actions_block_as_markdown(actions),
            SlackBlock::Context(context) => render_context_block_as_markdown(context),
            SlackBlock::Input(input) => render_input_block_as_markdown(input),
            SlackBlock::File(file) => render_file_block_as_markdown(file),
            SlackBlock::RichText(json_value) => render_rich_text_block_as_markdown(json_value),
            SlackBlock::Event(json_value) => render_event_block_as_markdown(json_value),
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn render_section_block_as_markdown(section: SlackSectionBlock) -> String {
    let mut result = vec![];
    if let Some(text) = section.text {
        result.push(render_text_block_as_markdown(text));
    }
    if let Some(fields) = section.fields {
        for field in fields {
            result.push(render_text_block_as_markdown(field));
        }
    }
    result.join("\n")
}

fn render_text_block_as_markdown(text: SlackBlockText) -> String {
    match text {
        SlackBlockText::Plain(plain) => render_plain_text_as_markdown(plain),
        SlackBlockText::MarkDown(markdown) => render_markdown_as_markdown(markdown),
    }
}

fn render_header_block_as_markdown(header: SlackHeaderBlock) -> String {
    format!("## {}", render_text_block_as_markdown(header.text.into()))
}

fn render_image_block_as_markdown(image: SlackImageBlock) -> String {
    format!("![{}]({})", image.alt_text, image.image_url)
}

fn render_actions_block_as_markdown(_actions: SlackActionsBlock) -> String {
    "".to_string()
}

fn render_context_block_as_markdown(context: SlackContextBlock) -> String {
    context
        .elements
        .into_iter()
        .map(|element| match element {
            SlackContextBlockElement::Image(image) => render_image_element_as_markdown(image),
            SlackContextBlockElement::Plain(text) => render_plain_text_as_markdown(text),
            SlackContextBlockElement::MarkDown(markdown) => render_markdown_as_markdown(markdown),
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn render_image_element_as_markdown(image: SlackBlockImageElement) -> String {
    format!("![{}]({})", image.alt_text, image.image_url)
}

fn render_plain_text_as_markdown(text: SlackBlockPlainText) -> String {
    text.text
}

fn render_markdown_as_markdown(markdown: SlackBlockMarkDownText) -> String {
    markdown.text
}

fn render_input_block_as_markdown(_input: SlackInputBlock) -> String {
    "".to_string()
}

fn render_file_block_as_markdown(_file: SlackFileBlock) -> String {
    "".to_string()
}

fn render_rich_text_block_as_markdown(json_value: serde_json::Value) -> String {
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
                    ) => render_rich_text_section_elements(elements),
                    (
                        Some(Some("rich_text_list")),
                        Some(serde_json::Value::String(style)),
                        Some(serde_json::Value::Array(elements)),
                    ) => render_rich_text_list_elements(elements, style),
                    (
                        Some(Some("rich_text_preformatted")),
                        None,
                        Some(serde_json::Value::Array(elements)),
                    ) => render_rich_text_preformatted_elements(elements),
                    (
                        Some(Some("rich_text_quote")),
                        None,
                        Some(serde_json::Value::Array(elements)),
                    ) => render_rich_text_quote_elements(elements),
                    _ => "".to_string(),
                }
            })
            .collect::<Vec<String>>()
            .join("\n"),
        _ => "".to_string(),
    }
}

fn render_rich_text_section_elements(elements: &Vec<serde_json::Value>) -> String {
    elements
        .iter()
        .map(render_rich_text_section_element)
        .collect::<Vec<String>>()
        .join("")
}

fn render_rich_text_list_elements(elements: &Vec<serde_json::Value>, style: &str) -> String {
    let list_style = if style == "ordered" { "1." } else { "-" };
    elements
        .iter()
        .map(|element| {
            format!(
                "{} {}",
                list_style,
                render_rich_text_section_element(element)
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn render_rich_text_preformatted_elements(elements: &Vec<serde_json::Value>) -> String {
    format!("```{}```", render_rich_text_section_elements(elements))
}

fn render_rich_text_quote_elements(elements: &Vec<serde_json::Value>) -> String {
    format!("> {}", render_rich_text_section_elements(elements))
}

fn render_rich_text_section_element(element: &serde_json::Value) -> String {
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
            let style = element.get("style");
            let channel_id = apply_bold_style(format!("#{channel_id}"), style);
            let channel_id = apply_italic_style(channel_id, style);
            let channel_id = apply_strike_style(channel_id, style);
            apply_code_style(channel_id, style)
        }
        Some(Some("user")) => {
            let Some(serde_json::Value::String(user_id)) = element.get("user_id") else {
                return "".to_string();
            };
            let style = element.get("style");
            let user_id = apply_bold_style(format!("@{user_id}"), style);
            let user_id = apply_italic_style(user_id, style);
            let user_id = apply_strike_style(user_id, style);
            apply_code_style(user_id, style)
        }
        Some(Some("usergroup")) => {
            let Some(serde_json::Value::String(usergroup_id)) = element.get("usergroup_id") else {
                return "".to_string();
            };
            let style = element.get("style");
            let usergroup_id = apply_bold_style(format!("@{usergroup_id}"), style);
            let usergroup_id = apply_italic_style(usergroup_id, style);
            let usergroup_id = apply_strike_style(usergroup_id, style);
            apply_code_style(usergroup_id, style)
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
            let Some(skin_tone) = splitted.get(1).map(|s| s.parse::<usize>().ok()).flatten() else {
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

fn render_event_block_as_markdown(_json_value: serde_json::Value) -> String {
    "".to_string()
}

fn render_url_as_markdown(url: &str, text: &str) -> String {
    format!("[{}]({})", text, url)
}

fn apply_bold_style(text: String, style: Option<&serde_json::Value>) -> String {
    if is_styled(style, "bold") {
        format!("**{}**", text)
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
        format!("~~{}~~", text)
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
        .map(|s| s.get(style_name).map(|b| b.as_bool()))
        .flatten()
        .flatten()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[test]
    fn test_empty_input() {
        assert_eq!(render_blocks_as_markdown(vec![]), "".to_string());
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
            render_blocks_as_markdown(blocks),
            "![Image](https://example.com/image.png)\n![Image2](https://example.com/image2.png)"
                .to_string()
        );
    }

    #[test]
    fn test_with_divider() {
        let blocks = vec![
            SlackBlock::Divider(SlackDividerBlock::new()),
            SlackBlock::Divider(SlackDividerBlock::new()),
        ];
        assert_eq!(render_blocks_as_markdown(blocks), "---\n---".to_string());
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
        assert_eq!(render_blocks_as_markdown(blocks), "".to_string());
    }

    #[test]
    fn test_with_action() {
        // No rendering
        let blocks = vec![SlackBlock::Actions(SlackActionsBlock::new(vec![]))];
        assert_eq!(render_blocks_as_markdown(blocks), "".to_string());
    }

    #[test]
    fn test_with_file() {
        // No rendering
        let blocks = vec![SlackBlock::File(SlackFileBlock::new("external_id".into()))];
        assert_eq!(render_blocks_as_markdown(blocks), "".to_string());
    }

    #[test]
    fn test_with_event() {
        // No rendering
        let blocks = vec![SlackBlock::Event(serde_json::json!({}))];
        assert_eq!(render_blocks_as_markdown(blocks), "".to_string());
    }

    #[test]
    fn test_header() {
        let blocks = vec![SlackBlock::Header(SlackHeaderBlock::new("Text".into()))];
        assert_eq!(render_blocks_as_markdown(blocks), "## Text".to_string());
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
            assert_eq!(render_blocks_as_markdown(blocks), "Text\nText2".to_string());
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
            assert_eq!(render_blocks_as_markdown(blocks), "Text\nText2".to_string());
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
                render_blocks_as_markdown(blocks),
                "Text11\nText12\nText21\nText22".to_string()
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
                render_blocks_as_markdown(blocks),
                "Text1\nText11\nText12\nText2\nText21\nText22".to_string()
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
                render_blocks_as_markdown(blocks),
                "![Image](https://example.com/image.png)\n![Image2](https://example.com/image2.png)".to_string()
            );
        }

        #[test]
        fn test_with_plain_text() {
            let blocks = vec![SlackBlock::Context(SlackContextBlock::new(vec![
                SlackContextBlockElement::Plain(SlackBlockPlainText::new("Text".to_string())),
                SlackContextBlockElement::Plain(SlackBlockPlainText::new("Text2".to_string())),
            ]))];
            assert_eq!(render_blocks_as_markdown(blocks), "Text\nText2".to_string());
        }

        #[test]
        fn test_with_markdown() {
            let blocks = vec![SlackBlock::Context(SlackContextBlock::new(vec![
                SlackContextBlockElement::MarkDown(SlackBlockMarkDownText::new("Text".to_string())),
                SlackContextBlockElement::MarkDown(SlackBlockMarkDownText::new(
                    "Text2".to_string(),
                )),
            ]))];
            assert_eq!(render_blocks_as_markdown(blocks), "Text\nText2".to_string());
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
            assert_eq!(render_blocks_as_markdown(blocks), "\n".to_string());
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
                        render_blocks_as_markdown(blocks),
                        "Text111Text112\nText121Text122\nText211Text212\nText221Text222"
                            .to_string()
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
                    assert_eq!(render_blocks_as_markdown(blocks), "**Text**".to_string());
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
                    assert_eq!(render_blocks_as_markdown(blocks), "_Text_".to_string());
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
                    assert_eq!(render_blocks_as_markdown(blocks), "~~Text~~".to_string());
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
                    assert_eq!(render_blocks_as_markdown(blocks), "`Text`".to_string());
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
                        render_blocks_as_markdown(blocks),
                        "~~_**Text**_~~".to_string()
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
                    assert_eq!(render_blocks_as_markdown(blocks), "#C0123456".to_string());
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
                    assert_eq!(render_blocks_as_markdown(blocks), "@user1".to_string());
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
                    assert_eq!(render_blocks_as_markdown(blocks), "@group1".to_string());
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
                        render_blocks_as_markdown(blocks),
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
                    assert_eq!(render_blocks_as_markdown(blocks), "👋".to_string());
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
                    assert_eq!(render_blocks_as_markdown(blocks), "👋🏻".to_string());
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
                    assert_eq!(render_blocks_as_markdown(blocks), "👋".to_string());
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
                    assert_eq!(render_blocks_as_markdown(blocks), ":bbb:".to_string());
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
                    render_blocks_as_markdown(blocks),
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
                    render_blocks_as_markdown(blocks),
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
                    render_blocks_as_markdown(blocks),
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
                    render_blocks_as_markdown(blocks),
                    "> Text1Text2".to_string()
                );
            }
        }
    }
}
