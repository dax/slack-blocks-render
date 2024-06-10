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

```rust
use slack_morphism::prelude::*;

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
let markdown_text = render_blocks_as_markdown(blocks);
```

## License

This project is distributed under the terms of the Apache License (Version 2.0).

See [LICENSE](LICENSE)
