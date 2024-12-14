# slack-blocks-render

[![Crates.io Version](https://badgers.space/crates/version/slack-blocks-render)](https://crates.io/crates/slack-blocks-render)
[![Docs.rs Latest](https://badgers.space/badge/docs.rs/latest/blue)](https://docs.rs/slack-blocks-render)
[![Build Status](https://badgers.space/github/checks/dax/slack-blocks-render?label=build)](https://github.com/dax/slack-blocks-render/actions/workflows/build.yaml)

Slack blocks render is a Rust library to render [Slack blocks](https://api.slack.com/reference/block-kit) as Markdown.

## Usage

First, add the `slack_blocks_render` crate as a dependency:

```sh
cargo add slack_blocks_render
```

Slack blocks render uses the `slack_morphism` data model as input so you should also add it as a dependency:

```sh
cargo add slack_morphism
```

### Simple usage (without Slack references resolution)

```rust
use slack_morphism::prelude::*;
use slack_blocks_render::{render_blocks_as_markdown, SlackReferences};

let blocks: Vec<SlackBlock> = vec![
    SlackBlock::RichText(serde_json::json!({
        "type": "rich_text",
        "elements": [
            {
                "type": "rich_text_section",
                "elements": [
                    {
                        "type": "text",
                        "text": "Hello World"
                    }
                ]
            },
        ]
    })),
];
let markdown_text = render_blocks_as_markdown(blocks, SlackReferences::default(), None);
```

### Usage with Slack references resolution

Slack references resolution is useful when you want to resolve user ID, channel ID, or user group ID in the Slack blocks.
Here is an example on how to use it:

```rust
use slack_morphism::prelude::*;
use slack_blocks_render::{
  find_slack_references_in_blocks, render_blocks_as_markdown, SlackReferences
};

let blocks: Vec<SlackBlock> = vec![
    SlackBlock::RichText(serde_json::json!({
        "type": "rich_text",
        "elements": [
            {
                "type": "rich_text_section",
                "elements": [
                    {
                        "type": "text",
                        "text": "Hello "
                    }
                ]
            },
            {
                "type": "rich_text_section",
                "elements": [
                    {
                        "type": "user",
                        "text": "U123456"
                    }
                ]
            },
        ]
    })),
];
// First, extract Slack references from the blocks
let slack_references = find_slack_references_in_blocks(&blocks);
// Then, resolve the references before rendering the blocks, this is on your own
// For example, you can use Slack API to resolve them
// ...
// let slack_user_ids = slack_references.users.keys().cloned().collect::<Vec<_>>();
// for slack_user_id in slack_user_ids {
//     let user_info = slack_api_client.users_info(slack_user_id).await?;
//     slack_references.users.insert(slack_user_id, user_info.name);
// }
// Finally, render the blocks as Markdown
let markdown_text = render_blocks_as_markdown(blocks, slack_references, None);
```

## License

This project is distributed under the terms of the Apache License (Version 2.0).

See [LICENSE](LICENSE)
