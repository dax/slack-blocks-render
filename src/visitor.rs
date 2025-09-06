use despatma::visitor;
use slack_morphism::prelude::*;

pub(crate) struct SlackEventBlock {
    #[allow(dead_code)]
    pub json_value: serde_json::Value,
}

pub(crate) struct SlackRichTextBlock {
    pub json_value: serde_json::Value,
}

visitor!(
    #[
        helper_tmpl = {
            match slack_block {
                SlackBlock::Section(section) => visitor.visit_slack_section_block(section),
                SlackBlock::Header(header) => visitor.visit_slack_header_block(header),
                SlackBlock::Divider(divider) => visitor.visit_slack_divider_block(divider),
                SlackBlock::Image(image) => visitor.visit_slack_image_block(image),
                SlackBlock::Actions(actions) => visitor.visit_slack_actions_block(actions),
                SlackBlock::Context(context) => visitor.visit_slack_context_block(context),
                SlackBlock::Input(input) => visitor.visit_slack_input_block(input),
                SlackBlock::File(file) => visitor.visit_slack_file_block(file),
                SlackBlock::Video(video) => visitor.visit_slack_video_block(video),
                SlackBlock::RichText(json_value) => visitor.visit_slack_rich_text_block(&SlackRichTextBlock { json_value: json_value.clone() }),
                SlackBlock::Event(json_value) => visitor.visit_slack_event_block(&SlackEventBlock { json_value: json_value.clone() }),
                SlackBlock::Markdown(markdown) => visitor.visit_slack_markdown_block(markdown),
                SlackBlock::ShareShortcut(_) => todo!()
            }
        },
    ]
    SlackBlock,
    #[
        helper_tmpl = {
            if let Some(text) = &slack_section_block.text {
                visitor.visit_slack_block_text(text);
            }
            if let Some(fields) = &slack_section_block.fields {
                for field in fields {
                    visitor.visit_slack_block_text(field);
                }
            }
        },
    ]
    SlackSectionBlock,
    #[
        helper_tmpl = {
            match slack_block_text {
                SlackBlockText::Plain(plain) => visitor.visit_slack_block_plain_text(plain),
                SlackBlockText::MarkDown(markdown) => visitor.visit_slack_block_mark_down_text(markdown),
            }
        },
    ]
    SlackBlockText,
    SlackBlockPlainText,
    #[
        helper_tmpl = {
            let slack_block_text = slack_header_block.text.clone().into();
            visitor.visit_slack_block_text(&slack_block_text);
        },
    ]
    SlackHeaderBlock,
    SlackDividerBlock,
    SlackImageBlock,
    SlackBlockImageElement,
    SlackActionsBlock,
    #[
        helper_tmpl = {
            for element in &slack_context_block.elements {
                match element {
                    SlackContextBlockElement::Image(image) => visitor.visit_slack_block_image_element(image),
                    SlackContextBlockElement::Plain(text) => visitor.visit_slack_block_plain_text(text),
                    SlackContextBlockElement::MarkDown(markdown) => visitor.visit_slack_block_mark_down_text(markdown),
                }
            }
        },
    ]
    SlackContextBlock,
    SlackBlockMarkDownText,
    SlackInputBlock,
    SlackFileBlock,
    SlackVideoBlock,
    SlackEventBlock,
    SlackRichTextBlock,
    SlackMarkdownBlock,
);
