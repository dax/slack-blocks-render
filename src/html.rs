use html_escape::encode_text;
use serde_json::Value;
use slack_morphism::prelude::*;

use crate::{
    references::SlackReferences,
    visitor::{
        visit_slack_block_image_element, visit_slack_block_mark_down_text,
        visit_slack_block_plain_text, visit_slack_context_block, visit_slack_divider_block,
        visit_slack_header_block, visit_slack_image_block, visit_slack_markdown_block,
        visit_slack_section_block, visit_slack_video_block, SlackRichTextBlock, Visitor,
    },
};

/// Render Slack's mrkdwn-formatted text directly as HTML.
/// Use this for raw Slack text (e.g., attachment text, plain text fallback)
/// that uses Slack's mrkdwn syntax but is not structured as blocks.
pub fn render_slack_mrkdwn_text_as_html(
    text: &str,
    slack_references: &SlackReferences,
    default_style_class: &str,
    highlight_style_class: &str,
) -> String {
    let renderer = HtmlRenderer::new(
        slack_references.clone(),
        default_style_class.to_string(),
        highlight_style_class.to_string(),
    );
    render_slack_mrkdwn_as_html(text, &renderer)
}

pub fn render_blocks_as_html(
    blocks: Vec<SlackBlock>,
    slack_references: SlackReferences,
    default_style_class: &str,
    highlight_style_class: &str,
) -> String {
    let mut block_renderer = HtmlRenderer::new(
        slack_references,
        default_style_class.to_string(),
        highlight_style_class.to_string(),
    );
    for block in blocks {
        block_renderer.visit_slack_block(&block);
    }
    block_renderer.sub_texts.join("")
}

struct HtmlRenderer {
    pub sub_texts: Vec<String>,
    pub slack_references: SlackReferences,
    pub default_style_class: String,
    pub highlight_style_class: String,
}

impl HtmlRenderer {
    pub fn new(
        slack_references: SlackReferences,
        default_style_class: String,
        highlight_style_class: String,
    ) -> Self {
        HtmlRenderer {
            sub_texts: vec![],
            slack_references,
            default_style_class,
            highlight_style_class,
        }
    }
}

impl Visitor for HtmlRenderer {
    fn visit_slack_section_block(&mut self, slack_section_block: &SlackSectionBlock) {
        let mut section_renderer = HtmlRenderer::new(
            self.slack_references.clone(),
            self.default_style_class.clone(),
            self.highlight_style_class.clone(),
        );
        visit_slack_section_block(&mut section_renderer, slack_section_block);
        let content = section_renderer.sub_texts.join("");
        if !content.is_empty() {
            self.sub_texts.push(format!("<p>{content}</p>\n"));
        }
    }

    fn visit_slack_block_plain_text(&mut self, slack_block_plain_text: &SlackBlockPlainText) {
        self.sub_texts
            .push(encode_text(&slack_block_plain_text.text).to_string());
        visit_slack_block_plain_text(self, slack_block_plain_text);
    }

    fn visit_slack_header_block(&mut self, slack_header_block: &SlackHeaderBlock) {
        let mut header_renderer = HtmlRenderer::new(
            self.slack_references.clone(),
            self.default_style_class.clone(),
            self.highlight_style_class.clone(),
        );
        visit_slack_header_block(&mut header_renderer, slack_header_block);
        self.sub_texts
            .push(format!("<h2>{}</h2>\n", header_renderer.sub_texts.join("")));
    }

    fn visit_slack_divider_block(&mut self, slack_divider_block: &SlackDividerBlock) {
        self.sub_texts.push("<hr />\n".to_string());
        visit_slack_divider_block(self, slack_divider_block);
    }

    fn visit_slack_image_block(&mut self, slack_image_block: &SlackImageBlock) {
        if let Some(image_url) = slack_image_block.image_url_or_file.image_url() {
            self.sub_texts.push(format!(
                "<p><img src=\"{image_url}\" alt=\"{}\" /></p>\n",
                encode_text(&slack_image_block.alt_text)
            ));
        }
        visit_slack_image_block(self, slack_image_block);
    }

    fn visit_slack_block_image_element(
        &mut self,
        slack_block_image_element: &SlackBlockImageElement,
    ) {
        if let Some(image_url) = slack_block_image_element.image_url_or_file.image_url() {
            self.sub_texts.push(format!(
                "<img src=\"{image_url}\" alt=\"{}\" />",
                encode_text(&slack_block_image_element.alt_text)
            ));
        }
        visit_slack_block_image_element(self, slack_block_image_element);
    }

    fn visit_slack_block_mark_down_text(
        &mut self,
        slack_block_mark_down_text: &SlackBlockMarkDownText,
    ) {
        self.sub_texts.push(render_slack_mrkdwn_as_html(
            &slack_block_mark_down_text.text,
            self,
        ));
        visit_slack_block_mark_down_text(self, slack_block_mark_down_text);
    }

    fn visit_slack_context_block(&mut self, slack_context_block: &SlackContextBlock) {
        let mut section_renderer = HtmlRenderer::new(
            self.slack_references.clone(),
            self.default_style_class.clone(),
            self.highlight_style_class.clone(),
        );
        visit_slack_context_block(&mut section_renderer, slack_context_block);
        let content = section_renderer.sub_texts.join("");
        if !content.is_empty() {
            self.sub_texts.push(format!("<p>{content}</p>\n"));
        }
    }

    fn visit_slack_rich_text_block(&mut self, slack_rich_text_block: &SlackRichTextBlock) {
        self.sub_texts.push(render_rich_text_block_as_html(
            slack_rich_text_block.json_value.clone(),
            self,
        ));
    }

    fn visit_slack_video_block(&mut self, slack_video_block: &SlackVideoBlock) {
        let title: SlackBlockText = slack_video_block.title.clone().into();
        let title = match title {
            SlackBlockText::Plain(plain_text) => plain_text.text,
            SlackBlockText::MarkDown(md_text) => md_text.text,
        };
        let escaped_title = encode_text(&title);
        if let Some(ref title_url) = slack_video_block.title_url {
            self.sub_texts.push(format!(
                "<p><em><a target=\"_blank\" rel=\"noopener noreferrer\" href=\"{title_url}\">{escaped_title}</a></em></p>\n"
            ));
        } else {
            self.sub_texts
                .push(format!("<p><em>{escaped_title}</em></p>\n"));
        }

        if let Some(description) = slack_video_block.description.clone() {
            let description: SlackBlockText = description.into();
            let description = match description {
                SlackBlockText::Plain(plain_text) => plain_text.text,
                SlackBlockText::MarkDown(md_text) => md_text.text,
            };
            self.sub_texts
                .push(format!("<p>{}</p>\n", encode_text(&description)));
        }

        self.sub_texts.push(format!(
            "<p><img src=\"{}\" alt=\"{}\" /></p>\n",
            slack_video_block.thumbnail_url,
            encode_text(&slack_video_block.alt_text)
        ));

        visit_slack_video_block(self, slack_video_block);
    }

    fn visit_slack_markdown_block(&mut self, slack_markdown_block: &SlackMarkdownBlock) {
        self.sub_texts.push(format!(
            "<p>{}</p>\n",
            encode_text(&slack_markdown_block.text)
        ));
        visit_slack_markdown_block(self, slack_markdown_block);
    }
}

// --- Rich text rendering ---

struct ListItem {
    content: String,
    indent: usize,
    style: String,
}

fn render_rich_text_block_as_html(
    json_value: serde_json::Value,
    renderer: &HtmlRenderer,
) -> String {
    let Some(serde_json::Value::Array(elements)) = json_value.get("elements") else {
        return String::new();
    };

    let mut result: Vec<String> = Vec::new();
    let mut list_accumulator: Vec<ListItem> = Vec::new();

    for element in elements {
        let elem_type = element.get("type").and_then(|t| t.as_str());

        if elem_type == Some("rich_text_list") {
            if let (Some(serde_json::Value::String(style)), Some(serde_json::Value::Array(items))) =
                (element.get("style"), element.get("elements"))
            {
                let indent: usize =
                    element.get("indent").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                for item in items {
                    if let Some(serde_json::Value::Array(inner)) = item.get("elements") {
                        list_accumulator.push(ListItem {
                            content: render_rich_text_section_elements(inner, renderer, true),
                            indent,
                            style: style.clone(),
                        });
                    }
                }
            }
            continue;
        }

        // Non-list element: flush accumulated list items
        if !list_accumulator.is_empty() {
            result.push(build_nested_list_html(&list_accumulator));
            list_accumulator.clear();
        }

        match (elem_type, element.get("elements")) {
            (Some("rich_text_section"), Some(serde_json::Value::Array(elems))) => {
                let content = render_rich_text_section_elements(elems, renderer, true);
                if !content.is_empty() {
                    result.push(format!("<p>{content}</p>\n"));
                }
            }
            (Some("rich_text_preformatted"), Some(serde_json::Value::Array(elems))) => {
                result.push(render_rich_text_preformatted_elements(elems, renderer));
            }
            (Some("rich_text_quote"), Some(serde_json::Value::Array(elems))) => {
                result.push(render_rich_text_quote_elements(elems, renderer));
            }
            _ => {}
        }
    }

    // Flush remaining list items
    if !list_accumulator.is_empty() {
        result.push(build_nested_list_html(&list_accumulator));
    }

    result.join("")
}

fn render_rich_text_section_elements(
    elements: &[serde_json::Value],
    renderer: &HtmlRenderer,
    fix_newlines_in_text: bool,
) -> String {
    let parts: Vec<(String, Option<StyleSet>)> = elements
        .iter()
        .map(|e| render_rich_text_section_element(e, renderer))
        .collect();

    let result = join_html(parts);
    if fix_newlines_in_text {
        fix_newlines(result)
    } else {
        result
    }
}

#[derive(Clone, PartialEq, Eq)]
struct StyleSet {
    bold: bool,
    italic: bool,
    strike: bool,
    code: bool,
}

impl StyleSet {
    fn from_style(style: Option<&Value>) -> Self {
        StyleSet {
            bold: is_styled(style, "bold"),
            italic: is_styled(style, "italic"),
            strike: is_styled(style, "strike"),
            code: is_styled(style, "code"),
        }
    }

    fn is_empty(&self) -> bool {
        !self.bold && !self.italic && !self.strike && !self.code
    }
}

fn wrap_with_styles(text: String, styles: &StyleSet) -> String {
    let mut result = text;
    if styles.bold {
        result = format!("<strong>{result}</strong>");
    }
    if styles.italic {
        result = format!("<em>{result}</em>");
    }
    if styles.strike {
        result = format!("<del>{result}</del>");
    }
    if styles.code {
        result = format!("<code>{result}</code>");
    }
    result
}

/// Merge consecutive elements with identical styles before wrapping.
/// This produces `<strong>Hello World!</strong>` instead of
/// `<strong>Hello</strong><strong> </strong><strong>World!</strong>`.
fn join_html(parts: Vec<(String, Option<StyleSet>)>) -> String {
    if parts.is_empty() {
        return String::new();
    }

    let mut merged: Vec<(String, Option<StyleSet>)> = Vec::new();
    for (content, styles) in parts {
        if let Some(last) = merged.last_mut() {
            if last.1 == styles && styles.is_some() {
                last.0.push_str(&content);
                continue;
            }
        }
        merged.push((content, styles));
    }

    merged
        .into_iter()
        .map(|(content, styles)| match styles {
            Some(s) if !s.is_empty() => wrap_with_styles(content, &s),
            _ => content,
        })
        .collect::<Vec<String>>()
        .join("")
}

fn render_rich_text_section_element(
    element: &serde_json::Value,
    renderer: &HtmlRenderer,
) -> (String, Option<StyleSet>) {
    match element.get("type").map(|t| t.as_str()) {
        Some(Some("text")) => {
            let Some(serde_json::Value::String(text)) = element.get("text") else {
                return (String::new(), None);
            };
            let style = element.get("style");
            let styles = StyleSet::from_style(style);
            (encode_text(text).to_string(), Some(styles))
        }
        Some(Some("channel")) => {
            let Some(serde_json::Value::String(channel_id)) = element.get("channel_id") else {
                return (String::new(), None);
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
            let styles = StyleSet::from_style(style);
            (format!("#{}", encode_text(channel_rendered)), Some(styles))
        }
        Some(Some("user")) => {
            let Some(serde_json::Value::String(user_id)) = element.get("user_id") else {
                return (String::new(), None);
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
            let style_class = if renderer
                .slack_references
                .user_id_to_highlight
                .as_ref()
                .is_some_and(|id| id.0 == *user_id)
            {
                &renderer.highlight_style_class
            } else {
                &renderer.default_style_class
            };
            let style = element.get("style");
            let styles = StyleSet::from_style(style);
            // Mention is a raw HTML fragment — not mergeable with adjacent styled text
            let html = format!(
                "<span class=\"{style_class}\">@{}</span>",
                encode_text(user_rendered)
            );
            (wrap_with_styles(html, &styles), None)
        }
        Some(Some("usergroup")) => {
            let Some(serde_json::Value::String(usergroup_id)) = element.get("usergroup_id") else {
                return (String::new(), None);
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
            let style_class = if renderer
                .slack_references
                .usergroup_ids_to_highlight
                .as_ref()
                .is_some_and(|ids| ids.iter().any(|id| id.0 == *usergroup_id))
            {
                &renderer.highlight_style_class
            } else {
                &renderer.default_style_class
            };
            let style = element.get("style");
            let styles = StyleSet::from_style(style);
            let html = format!(
                "<span class=\"{style_class}\">@{}</span>",
                encode_text(usergroup_rendered)
            );
            (wrap_with_styles(html, &styles), None)
        }
        Some(Some("emoji")) => {
            let Some(serde_json::Value::String(name)) = element.get("name") else {
                return (String::new(), None);
            };
            let style = element.get("style");
            let styles = StyleSet::from_style(style);
            let html = render_emoji(
                &SlackEmojiName(name.to_string()),
                &renderer.slack_references,
            );
            (wrap_with_styles(html, &styles), None)
        }
        Some(Some("link")) => {
            let Some(serde_json::Value::String(url)) = element.get("url") else {
                return (String::new(), None);
            };
            let text = element
                .get("text")
                .and_then(|t| t.as_str())
                .unwrap_or(url.as_str());
            let style = element.get("style");
            let styles = StyleSet::from_style(style);
            let html = format!(
                "<a target=\"_blank\" rel=\"noopener noreferrer\" href=\"{url}\">{}</a>",
                encode_text(text)
            );
            (wrap_with_styles(html, &styles), None)
        }
        _ => (String::new(), None),
    }
}

fn render_emoji(emoji_name: &SlackEmojiName, slack_references: &SlackReferences) -> String {
    if let Some(Some(emoji)) = slack_references.emojis.get(emoji_name) {
        match emoji {
            SlackEmojiRef::Alias(alias) => {
                return render_emoji(alias, slack_references);
            }
            SlackEmojiRef::Url(url) => {
                return format!(
                    "<img class=\"slack-emoji\" src=\"{url}\" alt=\":{}:\" />",
                    encode_text(&emoji_name.0)
                );
            }
        }
    }
    let name = &emoji_name.0;

    let splitted = name.split("::skin-tone-").collect::<Vec<&str>>();
    let Some(first) = splitted.first() else {
        return format!(":{}:", encode_text(name));
    };
    let Some(emoji) = emojis::get_by_shortcode(first) else {
        return format!(":{}:", encode_text(name));
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

fn render_rich_text_preformatted_elements(
    elements: &[serde_json::Value],
    renderer: &HtmlRenderer,
) -> String {
    let content = render_rich_text_section_elements(elements, renderer, false);
    format!("<pre style=\"white-space: pre-wrap; word-break: break-word;\"><code>{content}\n</code></pre>\n")
}

fn render_rich_text_quote_elements(
    elements: &[serde_json::Value],
    renderer: &HtmlRenderer,
) -> String {
    let content = render_rich_text_section_elements(elements, renderer, true);
    format!("<blockquote>\n<p>{content}</p>\n</blockquote>\n")
}

// --- Nested list construction ---

fn build_nested_list_html(items: &[ListItem]) -> String {
    if items.is_empty() {
        return String::new();
    }
    build_list_at_indent(items, 0).0
}

/// Returns (html_string, number_of_items_consumed)
fn build_list_at_indent(items: &[ListItem], base_indent: usize) -> (String, usize) {
    if items.is_empty() {
        return (String::new(), 0);
    }

    let tag = if items[0].style == "ordered" {
        "ol"
    } else {
        "ul"
    };
    let mut html = format!("<{tag}>\n");
    let mut i = 0;

    while i < items.len() && items[i].indent >= base_indent {
        if items[i].indent > base_indent {
            // Sub-list: attach to the previous <li> (which was left unclosed)
            let (sub_html, consumed) = build_list_at_indent(&items[i..], items[i].indent);
            html.push_str(&sub_html);
            html.push_str("</li>\n");
            i += consumed;
        } else {
            // Same level item
            html.push_str(&format!("<li>{}", items[i].content));
            // Check if next item is a sub-list
            if i + 1 < items.len() && items[i + 1].indent > base_indent {
                html.push('\n');
                // Don't close <li> — sub-list will be attached
            } else {
                html.push_str("</li>\n");
            }
            i += 1;
        }
    }

    html.push_str(&format!("</{tag}>\n"));
    (html, i)
}

// --- Helpers ---

fn is_styled(style: Option<&serde_json::Value>, style_name: &str) -> bool {
    style
        .and_then(|s| s.get(style_name).map(|b| b.as_bool()))
        .flatten()
        .unwrap_or_default()
}

/// Render Slack's mrkdwn format as HTML.
/// Handles: *bold*, _italic_, `code`, ~strike~, <url|label> links, :emoji:, \n line breaks.
fn render_slack_mrkdwn_as_html(text: &str, renderer: &HtmlRenderer) -> String {
    let mut output = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_bold = false;
    let mut in_italic = false;
    let mut in_strike = false;
    let mut in_code = false;

    while i < len {
        let ch = chars[i];

        // Backtick code (highest priority — no formatting inside)
        if ch == '`' {
            if in_code {
                output.push_str("</code>");
                in_code = false;
            } else {
                output.push_str("<code>");
                in_code = true;
            }
            i += 1;
            continue;
        }

        // Inside code: escape everything, no formatting
        if in_code {
            output.push_str(&encode_char(ch));
            i += 1;
            continue;
        }

        // Slack link: <url|label> or <url>
        if ch == '<' {
            if let Some(end) = chars[i..].iter().position(|&c| c == '>') {
                let inner: String = chars[i + 1..i + end].iter().collect();
                // Check for special Slack references: <@U123>, <!subteam^S123>
                if inner.starts_with('@') || inner.starts_with('!') || inner.starts_with('#') {
                    // User/channel/subteam mention in mrkdwn — render as escaped text
                    output.push_str(&encode_text(&inner));
                } else if let Some(pipe_pos) = inner.find('|') {
                    let url = &inner[..pipe_pos];
                    let label = &inner[pipe_pos + 1..];
                    output.push_str(&format!(
                        "<a target=\"_blank\" rel=\"noopener noreferrer\" href=\"{}\">{}</a>",
                        url,
                        encode_text(label)
                    ));
                } else {
                    // URL without label
                    output.push_str(&format!(
                        "<a target=\"_blank\" rel=\"noopener noreferrer\" href=\"{inner}\">{}</a>",
                        encode_text(&inner)
                    ));
                }
                i += end + 1;
                continue;
            }
            // Not a valid Slack link, escape the <
            output.push_str("&lt;");
            i += 1;
            continue;
        }

        // Emoji shortcode: :name: (checked before _ to avoid italic inside emoji names)
        if ch == ':' {
            if let Some(end) = chars[i + 1..].iter().position(|&c| c == ':') {
                let name: String = chars[i + 1..i + 1 + end].iter().collect();
                // Valid emoji names: non-empty, no spaces, may contain letters/digits/underscores/hyphens
                if !name.is_empty() && !name.contains(' ') {
                    let emoji_html =
                        render_emoji(&SlackEmojiName(name.clone()), &renderer.slack_references);
                    // If render_emoji returned :name: unchanged, it wasn't resolved
                    // but it's still a valid emoji shortcode — preserve it as-is
                    output.push_str(&emoji_html);
                    i += end + 2; // skip past closing :
                    continue;
                }
            }
            // Not a valid emoji shortcode, output as literal
            output.push(':');
            i += 1;
            continue;
        }

        // Bold: *text*
        if ch == '*' {
            if in_bold {
                output.push_str("</strong>");
                in_bold = false;
            } else {
                output.push_str("<strong>");
                in_bold = true;
            }
            i += 1;
            continue;
        }

        // Italic: _text_
        if ch == '_' {
            if in_italic {
                output.push_str("</em>");
                in_italic = false;
            } else {
                output.push_str("<em>");
                in_italic = true;
            }
            i += 1;
            continue;
        }

        // Strikethrough: ~text~
        if ch == '~' {
            if in_strike {
                output.push_str("</del>");
                in_strike = false;
            } else {
                output.push_str("<del>");
                in_strike = true;
            }
            i += 1;
            continue;
        }

        // Newline
        if ch == '\n' {
            output.push_str("<br />\n");
            i += 1;
            continue;
        }

        // Regular character — HTML escape
        output.push_str(&encode_char(ch));
        i += 1;
    }

    output
}

fn encode_char(ch: char) -> String {
    match ch {
        '&' => "&amp;".to_string(),
        '<' => "&lt;".to_string(),
        '>' => "&gt;".to_string(),
        '"' => "&quot;".to_string(),
        _ => ch.to_string(),
    }
}

fn fix_newlines(text: String) -> String {
    let result = text.replace('\t', "\u{2003}");
    let mut output = String::new();
    let mut chars = result.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\n' {
            output.push_str("<br />\n");
            while chars.peek() == Some(&' ') {
                chars.next();
                output.push_str("&nbsp;");
            }
        } else {
            output.push(ch);
        }
    }
    output.trim_end_matches("<br />\n").to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use url::Url;

    use super::*;
    use crate::test_utils::rich_text_block;

    fn render(blocks: Vec<SlackBlock>, refs: SlackReferences) -> String {
        render_blocks_as_html(blocks, refs, "text-primary", "text-accent")
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(render(vec![], SlackReferences::default()), "");
    }

    #[test]
    fn test_with_image() {
        let blocks = vec![
            SlackBlock::Image(SlackImageBlock::new(
                SlackImageUrlOrFile::ImageUrl {
                    image_url: Url::parse("https://example.com/image.png").unwrap(),
                },
                "Image".to_string(),
            )),
            SlackBlock::Image(SlackImageBlock::new(
                SlackImageUrlOrFile::ImageUrl {
                    image_url: Url::parse("https://example.com/image2.png").unwrap(),
                },
                "Image2".to_string(),
            )),
        ];
        assert_eq!(
            render(blocks, SlackReferences::default()),
            "<p><img src=\"https://example.com/image.png\" alt=\"Image\" /></p>\n\
             <p><img src=\"https://example.com/image2.png\" alt=\"Image2\" /></p>\n"
        );
    }

    #[test]
    fn test_with_divider() {
        let blocks = vec![
            SlackBlock::Divider(SlackDividerBlock::new()),
            SlackBlock::Divider(SlackDividerBlock::new()),
        ];
        assert_eq!(
            render(blocks, SlackReferences::default()),
            "<hr />\n<hr />\n"
        );
    }

    #[test]
    fn test_header() {
        let blocks = vec![SlackBlock::Header(SlackHeaderBlock::new("Text".into()))];
        assert_eq!(
            render(blocks, SlackReferences::default()),
            "<h2>Text</h2>\n"
        );
    }

    #[test]
    fn test_with_input() {
        let blocks = vec![SlackBlock::Input(SlackInputBlock::new(
            "label".into(),
            SlackInputBlockElement::PlainTextInput(SlackBlockPlainTextInputElement::new(
                "id".into(),
            )),
        ))];
        assert_eq!(render(blocks, SlackReferences::default()), "");
    }

    #[test]
    fn test_with_action() {
        let blocks = vec![SlackBlock::Actions(SlackActionsBlock::new(vec![]))];
        assert_eq!(render(blocks, SlackReferences::default()), "");
    }

    #[test]
    fn test_with_file() {
        let blocks = vec![SlackBlock::File(SlackFileBlock::new("external_id".into()))];
        assert_eq!(render(blocks, SlackReferences::default()), "");
    }

    #[test]
    fn test_with_event() {
        let blocks = vec![SlackBlock::Event(serde_json::json!({}))];
        assert_eq!(render(blocks, SlackReferences::default()), "");
    }

    #[test]
    fn test_with_video() {
        let blocks = vec![SlackBlock::Video(
            SlackVideoBlock::new(
                "alt text".into(),
                "Video title".into(),
                "https://example.com/thumbnail.jpg".parse().unwrap(),
                "https://example.com/video_embed.avi".parse().unwrap(),
            )
            .with_description("Video description".into())
            .with_title_url("https://example.com/video".parse().unwrap()),
        )];
        assert_eq!(
            render(blocks, SlackReferences::default()),
            "<p><em><a target=\"_blank\" rel=\"noopener noreferrer\" href=\"https://example.com/video\">Video title</a></em></p>\n\
             <p>Video description</p>\n\
             <p><img src=\"https://example.com/thumbnail.jpg\" alt=\"alt text\" /></p>\n"
        );
    }

    #[test]
    fn test_with_video_minimal() {
        let blocks = vec![SlackBlock::Video(SlackVideoBlock::new(
            "alt text".into(),
            "Video title".into(),
            "https://example.com/thumbnail.jpg".parse().unwrap(),
            "https://example.com/video_embed.avi".parse().unwrap(),
        ))];
        assert_eq!(
            render(blocks, SlackReferences::default()),
            "<p><em>Video title</em></p>\n\
             <p><img src=\"https://example.com/thumbnail.jpg\" alt=\"alt text\" /></p>\n"
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
                render(blocks, SlackReferences::default()),
                "<p>Text</p>\n<p>Text2</p>\n"
            );
        }

        #[test]
        fn test_with_fields() {
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_fields(
                vec![
                    SlackBlockText::Plain(SlackBlockPlainText::new("Text11".to_string())),
                    SlackBlockText::Plain(SlackBlockPlainText::new("Text12".to_string())),
                ],
            ))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p>Text11Text12</p>\n"
            );
        }
    }

    mod context {
        use super::*;

        #[test]
        fn test_with_image() {
            let blocks = vec![SlackBlock::Context(SlackContextBlock::new(vec![
                SlackContextBlockElement::Image(SlackBlockImageElement::new(
                    SlackImageUrlOrFile::ImageUrl {
                        image_url: Url::parse("https://example.com/image.png").unwrap(),
                    },
                    "Image".to_string(),
                )),
            ]))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p><img src=\"https://example.com/image.png\" alt=\"Image\" /></p>\n"
            );
        }

        #[test]
        fn test_with_plain_text() {
            let blocks = vec![SlackBlock::Context(SlackContextBlock::new(vec![
                SlackContextBlockElement::Plain(SlackBlockPlainText::new("Text".to_string())),
                SlackContextBlockElement::Plain(SlackBlockPlainText::new("Text2".to_string())),
            ]))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p>TextText2</p>\n"
            );
        }
    }

    mod rich_text {
        use super::*;

        #[test]
        fn test_with_empty_json() {
            let blocks = vec![rich_text_block(serde_json::json!({}))];
            assert_eq!(render(blocks, SlackReferences::default()), "");
        }

        mod rich_text_section {
            use super::*;

            #[test]
            fn test_with_text() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "text", "text": "Text111" },
                                { "type": "text", "text": "Text112" }
                            ]
                        },
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "text", "text": "Text211" },
                                { "type": "text", "text": "Text212" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<p>Text111Text112</p>\n<p>Text211Text212</p>\n"
                );
            }

            #[test]
            fn test_with_text_with_newline() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "text", "text": "Text1\nText2\n" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<p>Text1<br />\nText2</p>\n"
                );
            }

            #[test]
            fn test_with_text_with_only_newline() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "text", "text": "\n" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(render(blocks, SlackReferences::default()), "");
            }

            #[test]
            fn test_with_bold_text() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "text", "text": "Text", "style": { "bold": true } }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<p><strong>Text</strong></p>\n"
                );
            }

            #[test]
            fn test_with_consecutive_bold_text() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "text", "text": "Hello", "style": { "bold": true } },
                                { "type": "text", "text": " ", "style": { "bold": true } },
                                { "type": "text", "text": "World!", "style": { "bold": true } }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<p><strong>Hello World!</strong></p>\n"
                );
            }

            #[test]
            fn test_with_italic_text() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "text", "text": "Text", "style": { "italic": true } }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<p><em>Text</em></p>\n"
                );
            }

            #[test]
            fn test_with_strike_text() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "text", "text": "Text", "style": { "strike": true } }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<p><del>Text</del></p>\n"
                );
            }

            #[test]
            fn test_with_code_text() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "text", "text": "Text", "style": { "code": true } }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<p><code>Text</code></p>\n"
                );
            }

            #[test]
            fn test_with_all_styles() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                {
                                    "type": "text",
                                    "text": "Text",
                                    "style": { "bold": true, "italic": true, "strike": true, "code": true }
                                }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<p><code><del><em><strong>Text</strong></em></del></code></p>\n"
                );
            }

            #[test]
            fn test_with_link() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "link", "url": "https://example.com", "text": "Example" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<p><a target=\"_blank\" rel=\"noopener noreferrer\" href=\"https://example.com/\">Example</a></p>\n"
                );
            }

            #[test]
            fn test_with_user_mention() {
                let refs = SlackReferences {
                    users: HashMap::from([(
                        SlackUserId("U123".to_string()),
                        Some("john.doe".to_string()),
                    )]),
                    ..SlackReferences::default()
                };
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "user", "user_id": "U123" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, refs),
                    "<p><span class=\"text-primary\">@john.doe</span></p>\n"
                );
            }

            #[test]
            fn test_with_highlighted_user_mention() {
                let refs = SlackReferences {
                    users: HashMap::from([(
                        SlackUserId("U123".to_string()),
                        Some("john.doe".to_string()),
                    )]),
                    user_id_to_highlight: Some(SlackUserId("U123".to_string())),
                    ..SlackReferences::default()
                };
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "user", "user_id": "U123" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, refs),
                    "<p><span class=\"text-accent\">@john.doe</span></p>\n"
                );
            }

            #[test]
            fn test_with_usergroup_mention() {
                let refs = SlackReferences {
                    usergroups: HashMap::from([(
                        SlackUserGroupId("G123".to_string()),
                        Some("team-eng".to_string()),
                    )]),
                    ..SlackReferences::default()
                };
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "usergroup", "usergroup_id": "G123" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, refs),
                    "<p><span class=\"text-primary\">@team-eng</span></p>\n"
                );
            }

            #[test]
            fn test_with_highlighted_usergroup_mention() {
                let refs = SlackReferences {
                    usergroups: HashMap::from([(
                        SlackUserGroupId("G123".to_string()),
                        Some("team-eng".to_string()),
                    )]),
                    usergroup_ids_to_highlight: Some(vec![SlackUserGroupId("G123".to_string())]),
                    ..SlackReferences::default()
                };
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "usergroup", "usergroup_id": "G123" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, refs),
                    "<p><span class=\"text-accent\">@team-eng</span></p>\n"
                );
            }

            #[test]
            fn test_with_channel_ref() {
                let refs = SlackReferences {
                    channels: HashMap::from([(
                        SlackChannelId("C123".to_string()),
                        Some("general".to_string()),
                    )]),
                    ..SlackReferences::default()
                };
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "channel", "channel_id": "C123" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(render(blocks, refs), "<p>#general</p>\n");
            }

            #[test]
            fn test_with_unicode_emoji() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "emoji", "name": "wave" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<p>\u{1F44B}</p>\n"
                );
            }

            #[test]
            fn test_with_custom_emoji() {
                let refs = SlackReferences {
                    emojis: HashMap::from([(
                        SlackEmojiName("custom".to_string()),
                        Some(SlackEmojiRef::Url(
                            Url::parse("https://emoji.slack-edge.com/custom.png").unwrap(),
                        )),
                    )]),
                    ..SlackReferences::default()
                };
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "emoji", "name": "custom" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, refs),
                    "<p><img class=\"slack-emoji\" src=\"https://emoji.slack-edge.com/custom.png\" alt=\":custom:\" /></p>\n"
                );
            }
        }

        mod rich_text_section_with_inline_indentation {
            use super::*;

            #[test]
            fn test_with_tab_indentation() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "text", "text": "Title", "style": { "bold": true } },
                                { "type": "text", "text": "\n• item1\n\t• sub-item1\n• item2" }
                            ]
                        }
                    ]
                }))];
                let result = render(blocks, SlackReferences::default());
                assert!(
                    result.contains("\u{2003}"),
                    "Tab should be converted to em space, got: {result}"
                );
                assert!(
                    !result.contains('\t'),
                    "Raw tab should not remain in output, got: {result}"
                );
            }

            #[test]
            fn test_with_space_indentation() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "text", "text": "Title", "style": { "bold": true } },
                                { "type": "text", "text": "\n• " },
                                { "type": "text", "text": "eu-tools:", "style": { "bold": true } },
                                { "type": "text", "text": "\n     • standard\n\n• " },
                                { "type": "text", "text": "fr-api:", "style": { "bold": true } },
                                { "type": "text", "text": "\n     • document_parsing" }
                            ]
                        }
                    ]
                }))];
                let result = render(blocks, SlackReferences::default());
                // After <br />\n, leading spaces must be preserved as &nbsp;
                assert!(
                    result.contains("&nbsp;"),
                    "Leading spaces after line break should be converted to &nbsp;, got: {result}"
                );
                // The 5 leading spaces before sub-bullets should not collapse
                assert!(
                    result.contains("&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;•"),
                    "5 leading spaces should become 5 &nbsp; before the bullet, got: {result}"
                );
            }
        }

        mod rich_text_list {
            use super::*;

            #[test]
            fn test_ordered_list() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_list",
                            "style": "ordered",
                            "elements": [
                                {
                                    "type": "rich_text_section",
                                    "elements": [{ "type": "text", "text": "Item1" }]
                                },
                                {
                                    "type": "rich_text_section",
                                    "elements": [{ "type": "text", "text": "Item2" }]
                                }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<ol>\n<li>Item1</li>\n<li>Item2</li>\n</ol>\n"
                );
            }

            #[test]
            fn test_unordered_list() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_list",
                            "style": "bullet",
                            "elements": [
                                {
                                    "type": "rich_text_section",
                                    "elements": [{ "type": "text", "text": "Item1" }]
                                },
                                {
                                    "type": "rich_text_section",
                                    "elements": [{ "type": "text", "text": "Item2" }]
                                }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<ul>\n<li>Item1</li>\n<li>Item2</li>\n</ul>\n"
                );
            }

            #[test]
            fn test_nested_ordered_list() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_list",
                            "style": "ordered",
                            "elements": [
                                {
                                    "type": "rich_text_section",
                                    "elements": [{ "type": "text", "text": "Item1" }]
                                },
                                {
                                    "type": "rich_text_section",
                                    "elements": [{ "type": "text", "text": "Item2" }]
                                }
                            ]
                        },
                        {
                            "type": "rich_text_list",
                            "style": "ordered",
                            "indent": 1,
                            "elements": [
                                {
                                    "type": "rich_text_section",
                                    "elements": [{ "type": "text", "text": "Item2.1" }]
                                }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<ol>\n<li>Item1</li>\n<li>Item2\n<ol>\n<li>Item2.1</li>\n</ol>\n</li>\n</ol>\n"
                );
            }
        }

        mod rich_text_preformatted {
            use super::*;

            #[test]
            fn test_preformatted() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_preformatted",
                            "elements": [
                                { "type": "text", "text": "code here" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<pre style=\"white-space: pre-wrap; word-break: break-word;\"><code>code here\n</code></pre>\n"
                );
            }

            #[test]
            fn test_preformatted_with_newlines() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_preformatted",
                            "elements": [
                                { "type": "text", "text": "line1\nline2" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<pre style=\"white-space: pre-wrap; word-break: break-word;\"><code>line1\nline2\n</code></pre>\n"
                );
            }
        }

        mod rich_text_quote {
            use super::*;

            #[test]
            fn test_quote() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_quote",
                            "elements": [
                                { "type": "text", "text": "quoted text" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<blockquote>\n<p>quoted text</p>\n</blockquote>\n"
                );
            }

            #[test]
            fn test_quote_followed_by_text() {
                let blocks = vec![rich_text_block(serde_json::json!({
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_quote",
                            "elements": [
                                { "type": "text", "text": "quoted" }
                            ]
                        },
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "text", "text": "normal" }
                            ]
                        }
                    ]
                }))];
                assert_eq!(
                    render(blocks, SlackReferences::default()),
                    "<blockquote>\n<p>quoted</p>\n</blockquote>\n<p>normal</p>\n"
                );
            }
        }
    }

    #[test]
    fn test_html_escaping() {
        let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
            SlackBlockText::Plain(SlackBlockPlainText::new(
                "<script>alert('xss')</script>".to_string(),
            )),
        ))];
        assert_eq!(
            render(blocks, SlackReferences::default()),
            "<p>&lt;script&gt;alert(\'xss\')&lt;/script&gt;</p>\n"
        );
    }

    mod mrkdwn {
        use super::*;

        #[test]
        fn test_bold() {
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
                SlackBlockText::MarkDown(SlackBlockMarkDownText::new("*bold text*".to_string())),
            ))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p><strong>bold text</strong></p>\n"
            );
        }

        #[test]
        fn test_italic() {
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
                SlackBlockText::MarkDown(SlackBlockMarkDownText::new("_italic text_".to_string())),
            ))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p><em>italic text</em></p>\n"
            );
        }

        #[test]
        fn test_code() {
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
                SlackBlockText::MarkDown(SlackBlockMarkDownText::new("`code`".to_string())),
            ))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p><code>code</code></p>\n"
            );
        }

        #[test]
        fn test_strike() {
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
                SlackBlockText::MarkDown(SlackBlockMarkDownText::new("~strike~".to_string())),
            ))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p><del>strike</del></p>\n"
            );
        }

        #[test]
        fn test_nested_code_in_bold() {
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
                SlackBlockText::MarkDown(SlackBlockMarkDownText::new("*`+0.79%`*".to_string())),
            ))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p><strong><code>+0.79%</code></strong></p>\n"
            );
        }

        #[test]
        fn test_link_with_label() {
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
                SlackBlockText::MarkDown(SlackBlockMarkDownText::new(
                    "<https://example.com|Example>".to_string(),
                )),
            ))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p><a target=\"_blank\" rel=\"noopener noreferrer\" href=\"https://example.com\">Example</a></p>\n"
            );
        }

        #[test]
        fn test_link_without_label() {
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
                SlackBlockText::MarkDown(SlackBlockMarkDownText::new(
                    "<https://example.com>".to_string(),
                )),
            ))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p><a target=\"_blank\" rel=\"noopener noreferrer\" href=\"https://example.com\">https://example.com</a></p>\n"
            );
        }

        #[test]
        fn test_unicode_emoji() {
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
                SlackBlockText::MarkDown(SlackBlockMarkDownText::new(
                    "hello :ok_hand:".to_string(),
                )),
            ))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p>hello \u{1F44C}</p>\n"
            );
        }

        #[test]
        fn test_custom_emoji() {
            let refs = SlackReferences {
                emojis: HashMap::from([(
                    SlackEmojiName("custom".to_string()),
                    Some(SlackEmojiRef::Url(
                        Url::parse("https://emoji.slack-edge.com/custom.png").unwrap(),
                    )),
                )]),
                ..SlackReferences::default()
            };
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
                SlackBlockText::MarkDown(SlackBlockMarkDownText::new("hello :custom:".to_string())),
            ))];
            assert_eq!(
                render(blocks, refs),
                "<p>hello <img class=\"slack-emoji\" src=\"https://emoji.slack-edge.com/custom.png\" alt=\":custom:\" /></p>\n"
            );
        }

        #[test]
        fn test_line_breaks() {
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
                SlackBlockText::MarkDown(SlackBlockMarkDownText::new("line1\nline2".to_string())),
            ))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p>line1<br />\nline2</p>\n"
            );
        }

        #[test]
        fn test_bullet_list() {
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
                SlackBlockText::MarkDown(SlackBlockMarkDownText::new(
                    "• item1\n• item2".to_string(),
                )),
            ))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p>\u{2022} item1<br />\n\u{2022} item2</p>\n"
            );
        }

        #[test]
        fn test_html_escaping_in_mrkdwn() {
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
                SlackBlockText::MarkDown(SlackBlockMarkDownText::new("a & b < c".to_string())),
            ))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p>a &amp; b &lt; c</p>\n"
            );
        }

        #[test]
        fn test_costory_real_world() {
            let blocks = vec![SlackBlock::Section(
                SlackSectionBlock::new().with_text(SlackBlockText::MarkDown(
                    SlackBlockMarkDownText::new(
                        "Cloud spend has remained stable *`+0.79%`* compared to the previous week.\nLast week, you've spent *`$60.09K`* :ok_hand:".to_string(),
                    ),
                )),
            )];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p>Cloud spend has remained stable <strong><code>+0.79%</code></strong> compared to the previous week.<br />\nLast week, you've spent <strong><code>$60.09K</code></strong> \u{1F44C}</p>\n"
            );
        }

        #[test]
        fn test_unresolved_emoji_kept_as_literal() {
            let blocks = vec![SlackBlock::Section(SlackSectionBlock::new().with_text(
                SlackBlockText::MarkDown(SlackBlockMarkDownText::new(
                    ":unknown_emoji:".to_string(),
                )),
            ))];
            assert_eq!(
                render(blocks, SlackReferences::default()),
                "<p>:unknown_emoji:</p>\n"
            );
        }
    }

    mod render_slack_mrkdwn_text {
        use super::*;

        #[test]
        fn test_link_rendered_as_html() {
            let result = render_slack_mrkdwn_text_as_html(
                "Check <https://example.com|this link>",
                &SlackReferences::default(),
                "text-primary",
                "text-warning",
            );
            assert_eq!(
                result,
                "Check <a target=\"_blank\" rel=\"noopener noreferrer\" href=\"https://example.com\">this link</a>"
            );
        }

        #[test]
        fn test_bold_and_code() {
            let result = render_slack_mrkdwn_text_as_html(
                "*bold* and `code`",
                &SlackReferences::default(),
                "text-primary",
                "text-warning",
            );
            assert_eq!(result, "<strong>bold</strong> and <code>code</code>");
        }

        #[test]
        fn test_plain_text_escaped() {
            let result = render_slack_mrkdwn_text_as_html(
                "a & b",
                &SlackReferences::default(),
                "text-primary",
                "text-warning",
            );
            assert_eq!(result, "a &amp; b");
        }
    }
}
