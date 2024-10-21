use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use slack_morphism::{blocks::SlackBlock, SlackChannelId, SlackUserGroupId, SlackUserId};

use crate::visitor::{visit_slack_rich_text_block, SlackRichTextBlock, Visitor};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SlackReferences {
    pub channels: HashMap<SlackChannelId, Option<String>>,
    pub users: HashMap<SlackUserId, Option<String>>,
    pub usergroups: HashMap<SlackUserGroupId, Option<String>>,
}

impl SlackReferences {
    pub fn new() -> SlackReferences {
        SlackReferences {
            channels: HashMap::new(),
            users: HashMap::new(),
            usergroups: HashMap::new(),
        }
    }

    pub fn extend(&mut self, other: SlackReferences) {
        self.users.extend(other.users);
        self.usergroups.extend(other.usergroups);
        self.channels.extend(other.channels);
    }

    pub fn is_empty(&self) -> bool {
        self.users.is_empty() && self.usergroups.is_empty() && self.channels.is_empty()
    }
}

impl Default for SlackReferences {
    fn default() -> Self {
        Self::new()
    }
}

struct SlackReferencesFinder {
    pub slack_references: SlackReferences,
}

impl SlackReferencesFinder {
    pub fn new() -> SlackReferencesFinder {
        SlackReferencesFinder {
            slack_references: SlackReferences::default(),
        }
    }
}

pub fn find_slack_references_in_blocks(blocks: &[SlackBlock]) -> SlackReferences {
    let mut finder = SlackReferencesFinder::new();
    for block in blocks {
        finder.visit_slack_block(block);
    }
    finder.slack_references
}

impl Visitor for SlackReferencesFinder {
    fn visit_slack_rich_text_block(&mut self, slack_rich_text_block: &SlackRichTextBlock) {
        find_slack_references_in_rich_text_block(
            slack_rich_text_block.json_value.clone(),
            &mut self.slack_references,
        );
        visit_slack_rich_text_block(self, slack_rich_text_block);
    }
}

fn find_slack_references_in_rich_text_block(
    json_value: serde_json::Value,
    slack_references: &mut SlackReferences,
) {
    let Some(serde_json::Value::Array(elements)) = json_value.get("elements") else {
        return;
    };

    for element in elements {
        match (
            element.get("type").map(|t| t.as_str()),
            element.get("style"),
            element.get("elements"),
        ) {
            (Some(Some("rich_text_section")), None, Some(serde_json::Value::Array(elements))) => {
                find_slack_references_in_rich_text_section_elements(elements, slack_references)
            }
            (Some(Some("rich_text_list")), _, Some(serde_json::Value::Array(elements))) => {
                find_slack_references_in_rich_text_list_elements(elements, slack_references)
            }
            (
                Some(Some("rich_text_preformatted")),
                None,
                Some(serde_json::Value::Array(elements)),
            ) => {
                find_slack_references_in_rich_text_preformatted_elements(elements, slack_references)
            }
            (Some(Some("rich_text_quote")), None, Some(serde_json::Value::Array(elements))) => {
                find_slack_references_in_rich_text_quote_elements(elements, slack_references)
            }
            _ => {}
        }
    }
}

fn find_slack_references_in_rich_text_section_elements(
    elements: &[serde_json::Value],
    slack_references: &mut SlackReferences,
) {
    for element in elements {
        find_slack_references_in_rich_text_section_element(element, slack_references);
    }
}

fn find_slack_references_in_rich_text_list_elements(
    elements: &[serde_json::Value],
    slack_references: &mut SlackReferences,
) {
    for element in elements {
        if let Some(serde_json::Value::Array(inner_elements)) = element.get("elements") {
            find_slack_references_in_rich_text_section_elements(inner_elements, slack_references)
        }
    }
}

fn find_slack_references_in_rich_text_preformatted_elements(
    elements: &[serde_json::Value],
    slack_references: &mut SlackReferences,
) {
    find_slack_references_in_rich_text_section_elements(elements, slack_references);
}

fn find_slack_references_in_rich_text_quote_elements(
    elements: &[serde_json::Value],
    slack_references: &mut SlackReferences,
) {
    find_slack_references_in_rich_text_section_elements(elements, slack_references);
}

fn find_slack_references_in_rich_text_section_element(
    element: &serde_json::Value,
    slack_references: &mut SlackReferences,
) {
    match element.get("type").map(|t| t.as_str()) {
        Some(Some("channel")) => {
            let Some(serde_json::Value::String(channel_id)) = element.get("channel_id") else {
                return;
            };
            slack_references
                .channels
                .insert(SlackChannelId(channel_id.to_string()), None);
        }
        Some(Some("user")) => {
            let Some(serde_json::Value::String(user_id)) = element.get("user_id") else {
                return;
            };
            slack_references
                .users
                .insert(SlackUserId(user_id.to_string()), None);
        }
        Some(Some("usergroup")) => {
            let Some(serde_json::Value::String(usergroup_id)) = element.get("usergroup_id") else {
                return;
            };
            slack_references
                .usergroups
                .insert(SlackUserGroupId(usergroup_id.to_string()), None);
        }
        _ => {}
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_slack_references_with_user_id() {
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
            find_slack_references_in_blocks(&blocks),
            SlackReferences {
                users: HashMap::from([(SlackUserId("user1".to_string()), None)]),
                ..SlackReferences::default()
            }
        );
    }

    #[test]
    fn test_find_slack_references_with_usergroup_id() {
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
            find_slack_references_in_blocks(&blocks),
            SlackReferences {
                usergroups: HashMap::from([(SlackUserGroupId("group1".to_string()), None)]),
                ..SlackReferences::default()
            }
        );
    }

    #[test]
    fn test_find_slack_references_with_channel_id() {
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
            find_slack_references_in_blocks(&blocks),
            SlackReferences {
                channels: HashMap::from([(SlackChannelId("C0123456".to_string()), None)]),
                ..SlackReferences::default()
            }
        );
    }

    #[test]
    fn test_find_slack_references_with_multiple_references() {
        let blocks = vec![SlackBlock::RichText(serde_json::json!({
            "type": "rich_text",
            "elements": [
                {
                    "type": "rich_text_section",
                    "elements": [
                        {
                            "type": "user",
                            "user_id": "user1"
                        },
                        {
                            "type": "channel",
                            "channel_id": "C1234567"
                        },
                        {
                            "type": "usergroup",
                            "usergroup_id": "group1"
                        }
                    ]
                },
                {
                    "type": "rich_text_section",
                    "elements": [
                        {
                            "type": "user",
                            "user_id": "user2"
                        },
                        {
                            "type": "channel",
                            "channel_id": "C0123456"
                        },
                        {
                            "type": "usergroup",
                            "usergroup_id": "group2"
                        }
                    ]
                },
            ]
        }))];
        assert_eq!(
            find_slack_references_in_blocks(&blocks),
            SlackReferences {
                channels: HashMap::from([
                    (SlackChannelId("C0123456".to_string()), None),
                    (SlackChannelId("C1234567".to_string()), None)
                ]),
                users: HashMap::from([
                    (SlackUserId("user1".to_string()), None),
                    (SlackUserId("user2".to_string()), None)
                ]),
                usergroups: HashMap::from([
                    (SlackUserGroupId("group1".to_string()), None),
                    (SlackUserGroupId("group2".to_string()), None)
                ]),
            }
        );
    }
}
