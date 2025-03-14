# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/dax/slack-blocks-render/compare/v0.3.0...v0.4.0) - 2025-03-14

### Added

- Render custom Slack emojis

### Fixed

- Add support for `video` Slack block
- Use typed emojis references
- Render links without text as URL
- Indent nested list items
- Resolve custom emoji as alias
- Use CommonMark newline syntax ending lines with `\`
- Apply styles on emoji blocks
- Wrap preformatted text with new lines
- Add newline at the end of quoted text

### Other

- Update dependencies

## [0.3.0](https://github.com/dax/slack-blocks-render/compare/v0.2.5...v0.3.0) - 2024-12-14

### Added

- Add an option to add a custom delimiter for handles

## [0.2.5](https://github.com/dax/slack-blocks-render/compare/v0.2.4...v0.2.5) - 2024-12-13

### Fixed

- Add new lines between blocks and rich text elements

## [0.2.4](https://github.com/dax/slack-blocks-render/compare/v0.2.3...v0.2.4) - 2024-12-09

### Fixed

- Merge identical consecutive styling

## [0.2.3](https://github.com/dax/slack-blocks-render/compare/v0.2.2...v0.2.3) - 2024-12-07

### Fixed

- Update dependencies

## [0.2.2](https://github.com/dax/slack-blocks-render/compare/v0.2.1...v0.2.2) - 2024-12-07

### Fixed

- Update `slack-morphism` with `SlackUserGroup.user_count` type fix

## [0.2.1](https://github.com/dax/slack-blocks-render/compare/v0.2.0...v0.2.1) - 2024-10-21

### Added

- Render Slack blocks as raw text

## [0.2.0](https://github.com/dax/slack-blocks-render/compare/v0.1.1...v0.2.0) - 2024-10-21

### Added

- Resolve Slack user, channel and usergroup ID while rendering

## [0.1.1](https://github.com/dax/slack-blocks-render/compare/v0.1.0...v0.1.1) - 2024-06-10

### Fixed
- Fix `rich_text_list` block parsing
- Use single char for `bold` and `strike` styles
- Remove unexpected newline chars

### Other
- release

## [0.1.0](https://github.com/dax/slack-blocks-render/releases/tag/v0.1.0) - 2024-06-10

### Added
- Render Slack blocks as Markdown

### Other
- Initialize release-plz

