use serde_json::Value;
use slack_morphism::prelude::*;
use url::Url;

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
    handle_delimiter: Option<String>,
) -> String {
    let mut block_renderer = MarkdownRenderer::new(slack_references, handle_delimiter);
    for block in blocks {
        block_renderer.visit_slack_block(&block);
    }
    block_renderer.sub_texts.join("\n")
}

struct MarkdownRenderer {
    pub sub_texts: Vec<String>,
    pub slack_references: SlackReferences,
    pub handle_delimiter: Option<String>,
}

impl MarkdownRenderer {
    pub fn new(slack_references: SlackReferences, handle_delimiter: Option<String>) -> Self {
        MarkdownRenderer {
            sub_texts: vec![],
            slack_references,
            handle_delimiter,
        }
    }
}

fn join(mut texts: Vec<String>, join_str: &str) -> String {
    for i in 0..texts.len() {
        if i < texts.len() - 1 {
            if texts[i].ends_with('`') && texts[i + 1].starts_with('`') {
                texts[i].pop();
                texts[i + 1].remove(0);
            }
            if texts[i].ends_with('~') && texts[i + 1].starts_with('~') {
                texts[i].pop();
                texts[i + 1].remove(0);
            }
            if texts[i].ends_with('_') && texts[i + 1].starts_with('_') {
                texts[i].pop();
                texts[i + 1].remove(0);
            }
            if texts[i].ends_with('*') && texts[i + 1].starts_with('*') {
                texts[i].pop();
                texts[i + 1].remove(0);
            }
            if texts[i].starts_with("> ") && !texts[i + 1].starts_with("> ") {
                texts[i].push('\n');
            }
        }
    }
    texts.join(join_str)
}

impl Visitor for MarkdownRenderer {
    fn visit_slack_section_block(&mut self, slack_section_block: &SlackSectionBlock) {
        let mut section_renderer =
            MarkdownRenderer::new(self.slack_references.clone(), self.handle_delimiter.clone());
        visit_slack_section_block(&mut section_renderer, slack_section_block);
        self.sub_texts.push(join(section_renderer.sub_texts, ""));
    }

    fn visit_slack_block_plain_text(&mut self, slack_block_plain_text: &SlackBlockPlainText) {
        self.sub_texts.push(slack_block_plain_text.text.clone());
        visit_slack_block_plain_text(self, slack_block_plain_text);
    }

    fn visit_slack_header_block(&mut self, slack_header_block: &SlackHeaderBlock) {
        let mut header_renderer =
            MarkdownRenderer::new(self.slack_references.clone(), self.handle_delimiter.clone());
        visit_slack_header_block(&mut header_renderer, slack_header_block);
        self.sub_texts
            .push(format!("## {}", join(header_renderer.sub_texts, "")));
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
        let mut section_renderer =
            MarkdownRenderer::new(self.slack_references.clone(), self.handle_delimiter.clone());
        visit_slack_context_block(&mut section_renderer, slack_context_block);
        self.sub_texts.push(section_renderer.sub_texts.join(""));
    }

    fn visit_slack_rich_text_block(&mut self, slack_rich_text_block: &SlackRichTextBlock) {
        self.sub_texts.push(render_rich_text_block_as_markdown(
            slack_rich_text_block.json_value.clone(),
            self,
        ));
    }
}

fn render_rich_text_block_as_markdown(
    json_value: serde_json::Value,
    renderer: &MarkdownRenderer,
) -> String {
    match json_value.get("elements") {
        Some(serde_json::Value::Array(elements)) => join(
            elements
                .iter()
                .map(|element| {
                    match (
                        element.get("type").map(|t| t.as_str()),
                        element.get("style"),
                        element.get("elements"),
                        element.get("indent"),
                    ) {
                        (
                            Some(Some("rich_text_section")),
                            _,
                            Some(serde_json::Value::Array(elements)),
                            _,
                        ) => render_rich_text_section_elements(elements, renderer, true),
                        (
                            Some(Some("rich_text_list")),
                            Some(serde_json::Value::String(style)),
                            Some(serde_json::Value::Array(elements)),
                            Some(serde_json::Value::Number(indent)),
                        ) => render_rich_text_list_elements(
                            elements,
                            style,
                            indent
                                .as_u64()
                                .unwrap_or_default()
                                .try_into()
                                .unwrap_or_default(),
                            renderer,
                        ),
                        (
                            Some(Some("rich_text_list")),
                            Some(serde_json::Value::String(style)),
                            Some(serde_json::Value::Array(elements)),
                            _,
                        ) => render_rich_text_list_elements(elements, style, 0, renderer),
                        (
                            Some(Some("rich_text_preformatted")),
                            _,
                            Some(serde_json::Value::Array(elements)),
                            _,
                        ) => render_rich_text_preformatted_elements(elements, renderer),

                        (
                            Some(Some("rich_text_quote")),
                            _,
                            Some(serde_json::Value::Array(elements)),
                            _,
                        ) => render_rich_text_quote_elements(elements, renderer),

                        _ => "".to_string(),
                    }
                })
                .collect::<Vec<String>>(),
            "\n",
        ),
        _ => "".to_string(),
    }
}

fn render_rich_text_section_elements(
    elements: &[serde_json::Value],
    renderer: &MarkdownRenderer,
    fix_newlines_in_text: bool,
) -> String {
    let result = join(
        elements
            .iter()
            .map(|e| render_rich_text_section_element(e, renderer))
            .collect::<Vec<String>>(),
        "",
    );
    if fix_newlines_in_text {
        fix_newlines(result)
    } else {
        result
    }
}

fn render_rich_text_list_elements(
    elements: &[serde_json::Value],
    style: &str,
    indent: usize,
    renderer: &MarkdownRenderer,
) -> String {
    let list_style = if style == "ordered" { "1." } else { "-" };
    let space_per_level = if style == "ordered" { 3 } else { 2 };
    let indent_prefix = " ".repeat(space_per_level * indent);
    elements
        .iter()
        .filter_map(|element| {
            if let Some(serde_json::Value::Array(elements)) = element.get("elements") {
                Some(render_rich_text_section_elements(elements, renderer, true))
            } else {
                None
            }
        })
        .map(|element| format!("{indent_prefix}{list_style} {element}"))
        .collect::<Vec<String>>()
        .join("\n")
}

fn render_rich_text_preformatted_elements(
    elements: &[serde_json::Value],
    renderer: &MarkdownRenderer,
) -> String {
    format!(
        "```\n{}\n```",
        render_rich_text_section_elements(elements, renderer, false)
    )
}

fn render_rich_text_quote_elements(
    elements: &[serde_json::Value],
    renderer: &MarkdownRenderer,
) -> String {
    format!(
        "> {}",
        render_rich_text_section_elements(elements, renderer, true)
    )
}

fn render_rich_text_section_element(
    element: &serde_json::Value,
    renderer: &MarkdownRenderer,
) -> String {
    let handle_delimiter = renderer.handle_delimiter.clone().unwrap_or_default();
    match element.get("type").map(|t| t.as_str()) {
        Some(Some("text")) => {
            let Some(serde_json::Value::String(text)) = element.get("text") else {
                return "".to_string();
            };
            let style = element.get("style");
            apply_all_styles(text.to_string(), style)
        }
        Some(Some("channel")) => {
            let Some(serde_json::Value::String(channel_id)) = element.get("channel_id") else {
                return "".to_string();
            };
            let channel_rendered = if let Some(Some(channel_name)) = renderer
                .slack_references
                .channels
                .get(&SlackChannelId(channel_id.clone()))
            {
                channel_name
            } else {
                channel_id
            };
            let style = element.get("style");
            apply_all_styles(format!("#{channel_rendered}"), style)
        }
        Some(Some("user")) => {
            let Some(serde_json::Value::String(user_id)) = element.get("user_id") else {
                return "".to_string();
            };
            let user_rendered = if let Some(Some(user_name)) = renderer
                .slack_references
                .users
                .get(&SlackUserId(user_id.clone()))
            {
                user_name
            } else {
                user_id
            };
            let style = element.get("style");
            apply_all_styles(
                format!("{handle_delimiter}@{user_rendered}{handle_delimiter}"),
                style,
            )
        }
        Some(Some("usergroup")) => {
            let Some(serde_json::Value::String(usergroup_id)) = element.get("usergroup_id") else {
                return "".to_string();
            };
            let usergroup_rendered = if let Some(Some(usergroup_name)) = renderer
                .slack_references
                .usergroups
                .get(&SlackUserGroupId(usergroup_id.clone()))
            {
                usergroup_name
            } else {
                usergroup_id
            };
            let style = element.get("style");
            apply_all_styles(
                format!("{handle_delimiter}@{usergroup_rendered}{handle_delimiter}"),
                style,
            )
        }
        Some(Some("emoji")) => {
            let Some(serde_json::Value::String(name)) = element.get("name") else {
                return "".to_string();
            };
            let style = element.get("style");
            render_emoji(name, &renderer.slack_references, style)
        }
        Some(Some("link")) => {
            let Some(serde_json::Value::String(url)) = element.get("url") else {
                return "".to_string();
            };
            let Some(serde_json::Value::String(text)) = element.get("text") else {
                return render_url_as_markdown(url, url);
            };
            let style = element.get("style");
            apply_all_styles(render_url_as_markdown(url, text), style)
        }
        _ => "".to_string(),
    }
}

fn render_url_as_markdown(url: &str, text: &str) -> String {
    format!("[{}]({})", text, url)
}

fn render_emoji(name: &str, slack_references: &SlackReferences, style: Option<&Value>) -> String {
    let name = if let Some(Some(emoji)) = slack_references.emojis.get(name) {
        if emoji.starts_with("alias:") {
            return render_emoji(emoji.trim_start_matches("alias:"), slack_references, style);
        } else if emoji.parse::<Url>().is_ok() {
            return apply_all_styles(format!("![:{}:]({})", name, emoji), style);
        } else {
            emoji
        }
    } else {
        name
    };

    let splitted = name.split("::skin-tone-").collect::<Vec<&str>>();
    let Some(first) = splitted.first() else {
        return apply_all_styles(format!(":{}:", name), style);
    };
    let Some(emoji) = emojis::get_by_shortcode(first) else {
        return apply_all_styles(format!(":{}:", name), style);
    };
    let Some(skin_tone) = splitted.get(1).and_then(|s| s.parse::<usize>().ok()) else {
        return apply_all_styles(emoji.to_string(), style);
    };
    let Some(mut skin_tones) = emoji.skin_tones() else {
        return apply_all_styles(emoji.to_string(), style);
    };
    let Some(skinned_emoji) = skin_tones.nth(skin_tone - 1) else {
        return apply_all_styles(emoji.to_string(), style);
    };
    apply_all_styles(skinned_emoji.to_string(), style)
}

fn apply_all_styles(text: String, style: Option<&serde_json::Value>) -> String {
    let text = apply_bold_style(text, style);
    let text = apply_italic_style(text, style);
    let text = apply_strike_style(text, style);
    apply_code_style(text, style)
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

fn fix_newlines(text: String) -> String {
    text.replace("\n", "\\\n")
        .trim_end_matches("\\\n")
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use url::Url;

    use super::*;

    #[test]
    fn test_empty_input() {
        assert_eq!(
            render_blocks_as_markdown(vec![], SlackReferences::default(), None),
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
            render_blocks_as_markdown(blocks, SlackReferences::default(), None),
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
        assert_eq!(
            render_blocks_as_markdown(blocks, SlackReferences::default(), None),
            "---\n\n---\n".to_string()
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
            render_blocks_as_markdown(blocks, SlackReferences::default(), None),
            "".to_string()
        );
    }

    #[test]
    fn test_with_action() {
        // No rendering
        let blocks = vec![SlackBlock::Actions(SlackActionsBlock::new(vec![]))];
        assert_eq!(
            render_blocks_as_markdown(blocks, SlackReferences::default(), None),
            "".to_string()
        );
    }

    #[test]
    fn test_with_file() {
        // No rendering
        let blocks = vec![SlackBlock::File(SlackFileBlock::new("external_id".into()))];
        assert_eq!(
            render_blocks_as_markdown(blocks, SlackReferences::default(), None),
            "".to_string()
        );
    }

    #[test]
    fn test_with_event() {
        // No rendering
        let blocks = vec![SlackBlock::Event(serde_json::json!({}))];
        assert_eq!(
            render_blocks_as_markdown(blocks, SlackReferences::default(), None),
            "".to_string()
        );
    }

    #[test]
    fn test_header() {
        let blocks = vec![SlackBlock::Header(SlackHeaderBlock::new("Text".into()))];
        assert_eq!(
            render_blocks_as_markdown(blocks, SlackReferences::default(), None),
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
                render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                "Text\nText2".to_string()
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
                render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                "Text\nText2".to_string()
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
                render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                "Text11Text12\nText21Text22".to_string()
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
                render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                "Text1Text11Text12\nText2Text21Text22".to_string()
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
                render_blocks_as_markdown(blocks, SlackReferences::default(), None),
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
                render_blocks_as_markdown(blocks, SlackReferences::default(), None),
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
                render_blocks_as_markdown(blocks, SlackReferences::default(), None),
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
                render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                "\n".to_string()
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "Text111Text112\nText121Text122\nText211Text212\nText221Text222"
                            .to_string()
                    );
                }

                #[test]
                fn test_with_text_with_newline() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Text111\nText112\n"
                                    },
                                    {
                                        "type": "text",
                                        "text": "Text211\nText212\n"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "Text111\\\nText112\\\nText211\\\nText212".to_string()
                    );
                }

                #[test]
                fn test_with_text_with_newline_char() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Text111\\nText112"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "Text111\\nText112".to_string()
                    );
                }

                #[test]
                fn test_with_text_with_only_newline() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "\n"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "".to_string()
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "*Text*".to_string()
                    );
                }

                #[test]
                fn test_with_consecutive_bold_text() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Hello",
                                        "style": {
                                            "bold": true
                                        }
                                    },
                                    {
                                        "type": "text",
                                        "text": " ",
                                        "style": {
                                            "bold": true
                                        }
                                    },
                                    {
                                        "type": "text",
                                        "text": "World!",
                                        "style": {
                                            "bold": true
                                        }
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "*Hello World!*".to_string()
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "_Text_".to_string()
                    );
                }

                #[test]
                fn test_with_consecutive_italic_text() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Hello",
                                        "style": {
                                            "italic": true
                                        }
                                    },
                                    {
                                        "type": "text",
                                        "text": " ",
                                        "style": {
                                            "italic": true
                                        }
                                    },
                                    {
                                        "type": "text",
                                        "text": "World!",
                                        "style": {
                                            "italic": true
                                        }
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "_Hello World!_".to_string()
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "~Text~".to_string()
                    );
                }

                #[test]
                fn test_with_consecutive_strike_text() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Hello",
                                        "style": {
                                            "strike": true
                                        }
                                    },
                                    {
                                        "type": "text",
                                        "text": " ",
                                        "style": {
                                            "strike": true
                                        }
                                    },
                                    {
                                        "type": "text",
                                        "text": "World!",
                                        "style": {
                                            "strike": true
                                        }
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "~Hello World!~".to_string()
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "`Text`".to_string()
                    );
                }

                #[test]
                fn test_with_consecutive_code_text() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Text1",
                                        "style": {
                                            "code": true
                                        }
                                    },
                                    {
                                        "type": "text",
                                        "text": "Text2",
                                        "style": {
                                            "code": true
                                        }
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "`Text1Text2`".to_string()
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "~_*Text*_~".to_string()
                    );
                }

                #[test]
                fn test_with_consecutive_styled_text() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "name": "chart_with_upwards_trend",
                                        "type": "emoji",
                                        "style": {
                                            "bold": true,
                                            "italic": true,
                                            "strike": true
                                        },
                                    },
                                    {
                                        "type": "text",
                                        "text": "Hello",
                                        "style": {
                                            "bold": true,
                                            "italic": true,
                                            "strike": true
                                        }
                                    },
                                    {
                                        "type": "text",
                                        "text": " ",
                                        "style": {
                                            "bold": true,
                                            "italic": true,
                                            "strike": true
                                        }
                                    },
                                    {
                                        "type": "text",
                                        "text": "World!",
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "~_*📈Hello World!*_~".to_string()
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
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
                            },
                            None
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "@user1".to_string()
                    );
                }

                #[test]
                fn test_with_user_id_and_custom_delimiter() {
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
                            SlackReferences::default(),
                            Some("@".to_string())
                        ),
                        "@@user1@".to_string()
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
                            },
                            None
                        ),
                        "@John Doe".to_string()
                    );
                }

                #[test]
                fn test_with_user_id_and_reference_and_custom_delimiter() {
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
                            },
                            Some("@".to_string())
                        ),
                        "@@John Doe@".to_string()
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "@group1".to_string()
                    );
                }

                #[test]
                fn test_with_usergroup_id_and_custom_delimiter() {
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
                            SlackReferences::default(),
                            Some("@".to_string())
                        ),
                        "@@group1@".to_string()
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
                            },
                            None
                        ),
                        "@Admins".to_string()
                    );
                }

                #[test]
                fn test_with_usergroup_id_and_reference_and_custom_delimiter() {
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
                            },
                            Some("@".to_string())
                        ),
                        "@@Admins@".to_string()
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "[example](https://example.com)".to_string()
                    );
                }

                #[test]
                fn test_with_url_without_text() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "link",
                                        "url": "https://example.com"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        "[https://example.com](https://example.com)".to_string()
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
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
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
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
                                        "name": "unknown1"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                        ":unknown1:".to_string()
                    );
                }

                #[test]
                fn test_with_unknown_emoji_with_slack_reference_alias() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "emoji",
                                        "name": "unknown1"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(
                            blocks,
                            SlackReferences {
                                emojis: HashMap::from([(
                                    "unknown1".to_string(),
                                    Some("alias:wave".to_string())
                                )]),
                                ..SlackReferences::default()
                            },
                            None
                        ),
                        "👋".to_string()
                    );
                }

                #[test]
                fn test_with_unknown_emoji_with_slack_reference_alias_to_custom_emoji() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "emoji",
                                        "name": "unknown1"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(
                            blocks,
                            SlackReferences {
                                emojis: HashMap::from([
                                    ("unknown1".to_string(), Some("alias:unknown2".to_string())),
                                    (
                                        "unknown2".to_string(),
                                        Some("https://emoji.com/unknown2.png".to_string())
                                    )
                                ]),
                                ..SlackReferences::default()
                            },
                            None
                        ),
                        "![:unknown2:](https://emoji.com/unknown2.png)".to_string()
                    );
                }

                #[test]
                fn test_with_unknown_emoji_with_slack_reference_image_url() {
                    let blocks = vec![SlackBlock::RichText(serde_json::json!({
                        "type": "rich_text",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "emoji",
                                        "name": "unknown1"
                                    }
                                ]
                            }
                        ]
                    }))];
                    assert_eq!(
                        render_blocks_as_markdown(
                            blocks,
                            SlackReferences {
                                emojis: HashMap::from([(
                                    "unknown1".to_string(),
                                    Some("https://emoji.com/unknown1.png".to_string())
                                )]),
                                ..SlackReferences::default()
                            },
                            None
                        ),
                        "![:unknown1:](https://emoji.com/unknown1.png)".to_string()
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
                    render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                    "1. Text1\n1. Text2".to_string()
                );
            }

            #[test]
            fn test_with_nested_ordered_list() {
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
                            ]
                        },
                        {
                            "type": "rich_text_list",
                            "style": "ordered",
                            "elements": [
                                {
                                    "type": "rich_text_section",
                                    "elements": [
                                        {
                                            "type": "text",
                                            "text": "Text2"
                                        }
                                    ]
                                },
                            ]
                        },
                        {
                            "type": "rich_text_list",
                            "style": "ordered",
                            "indent": 1,
                            "elements": [
                                {
                                    "type": "rich_text_section",
                                    "elements": [
                                        {
                                            "type": "text",
                                            "text": "Text2.1"
                                        }
                                    ]
                                },
                            ]
                        },
                        {
                            "type": "rich_text_list",
                            "style": "ordered",
                            "indent": 2,
                            "elements": [
                                {
                                    "type": "rich_text_section",
                                    "elements": [
                                        {
                                            "type": "text",
                                            "text": "Text2.1.1"
                                        }
                                    ]
                                },
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                    "1. Text1\n1. Text2\n   1. Text2.1\n      1. Text2.1.1".to_string()
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
                    render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                    "- Text1\n- Text2".to_string()
                );
            }
        }

        #[test]
        fn test_with_nested_bullet_list() {
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
                        ]
                    },
                    {
                        "type": "rich_text_list",
                        "style": "bullet",
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Text2"
                                    }
                                ]
                            },
                        ]
                    },
                    {
                        "type": "rich_text_list",
                        "style": "bullet",
                        "indent": 1,
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Text2.1"
                                    }
                                ]
                            },
                        ]
                    },
                    {
                        "type": "rich_text_list",
                        "style": "bullet",
                        "indent": 2,
                        "elements": [
                            {
                                "type": "rich_text_section",
                                "elements": [
                                    {
                                        "type": "text",
                                        "text": "Text2.1.1"
                                    }
                                ]
                            },
                        ]
                    }
                ]
            }))];
            assert_eq!(
                render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                "- Text1\n- Text2\n  - Text2.1\n    - Text2.1.1".to_string()
            );
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
                    render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                    "```\nText1Text2\n```".to_string()
                );
            }

            #[test]
            fn test_with_text_and_newline() {
                let blocks = vec![SlackBlock::RichText(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_preformatted",
                            "elements": [
                                {
                                    "type": "text",
                                    "text": "test:\n  sub: value"
                                }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                    "```\ntest:\n  sub: value\n```".to_string()
                );
            }

            #[test]
            fn test_with_preformatted_text_followed_by_text() {
                let blocks = vec![SlackBlock::RichText(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_preformatted",
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
                        },
                    ]
                }))];
                assert_eq!(
                    render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                    "```\nText1\n```\nText2".to_string()
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
                    render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                    "> Text1Text2".to_string()
                );
            }

            #[test]
            fn test_with_quoted_text_followed_by_quoted_text() {
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
                            ]
                        },
                        {
                            "type": "rich_text_quote",
                            "elements": [
                                {
                                    "type": "text",
                                    "text": "Text2"
                                }
                            ]
                        },
                    ]
                }))];
                assert_eq!(
                    render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                    "> Text1\n> Text2".to_string()
                );
            }

            #[test]
            fn test_with_quoted_text_followed_by_non_quoted_text() {
                let blocks = vec![SlackBlock::RichText(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_quote",
                            "elements": [
                                {
                                    "text": "Text1",
                                    "type": "text"
                                }
                            ]
                        },
                        {
                            "type": "rich_text_section",
                            "elements": [
                                {
                                    "text": "Text2",
                                    "type": "text"
                                },
                            ]
                        }
                    ]
                }))];

                assert_eq!(
                    render_blocks_as_markdown(blocks, SlackReferences::default(), None),
                    "> Text1\n\nText2".to_string()
                );
            }
        }
    }
}
