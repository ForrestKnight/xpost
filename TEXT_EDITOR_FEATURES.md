# Text Editor Features

## Overview
The compose area in xpost now uses `tui-textarea`, providing a professional text editing experience with advanced features.

## Features Implemented

### Text Editing
- ✅ **Blinking cursor** - Visual cursor (stays visible)
- ✅ **Multi-line editing** - Full support for newlines and multi-paragraph posts
- ✅ **Text selection** - Keyboard-based selection with Shift+arrow keys
- ✅ **Undo/Redo** - Built-in undo/redo support (Ctrl+Z / Ctrl+Y)
- ✅ **Mouse wheel scrolling** - Scroll through text with mouse wheel
- ❌ **Mouse click/drag** - Not supported by tui-textarea (use Shift+arrows instead)

### Keyboard Shortcuts

#### Composing Mode
- **Ctrl+A** - Move to start of line (Emacs-style)
- **Ctrl+C** - Exit app
- **Ctrl+E** - Move to end of line (Emacs-style)
- **Ctrl+Shift+C** - Copy selected text
- **Ctrl+Shift+V** - Paste text from clipboard
- **Ctrl+X** - Cut selected text
- **Ctrl+K** - Delete to end of line (yank)
- **Ctrl+U** - Upload image from file path
- **Ctrl+S** - Save draft locally
- **Ctrl+D** - Open draft browser
- **Ctrl+P** - Post to X
- **Ctrl+Z** - Undo
- **Ctrl+Y** - Redo
- **Shift+Arrow keys** - Select text (hold Shift while using arrows)
- **Esc** - Exit app
- **Arrow keys** - Navigate text
- **Home/End** - Jump to start/end of line
- **Page Up/Down** - Scroll through text
- **Mouse wheel** - Scroll text

#### Draft Browser
- **↑/↓** - Navigate through saved drafts
- **Enter** - Load selected draft into compose area
- **Delete** - Remove selected draft
- **Esc** - Return to compose mode

### Draft Management
- **Auto-save location**: `~/.config/xpost/drafts/`
- **Draft format**: JSON files with timestamps
- **Draft preview**: Shows date and first 60 characters
- **Draft updates**: Re-saving an already loaded draft updates it instead of creating a new one
- **Persistent storage**: Drafts survive across sessions

## Usage Examples

### Creating and Saving a Draft
1. Type your message in the compose area
2. Press **Ctrl+S** to save as a draft
3. Continue editing or exit - the draft is saved

### Loading a Draft
1. Press **Ctrl+D** to open the draft browser
2. Use ↑/↓ to navigate through your saved drafts
3. Press **Enter** to load a draft
4. Edit and post, or save changes with **Ctrl+S**

### Deleting a Draft
1. Press **Ctrl+D** to open the draft browser
2. Navigate to the draft you want to delete
3. Press **Delete** to remove it

## Technical Details

### Dependencies Added
- `tui-textarea = "0.6"` - Professional text editing widget
- `chrono = { version = "0.4", features = ["serde"] }` - Timestamp handling

### File Structure
```
~/.config/xpost/
├── config.toml          # API credentials
└── drafts/              # Draft storage directory
    ├── 1732467123456.json
    ├── 1732467234567.json
    └── ...
```

### Draft Format
```json
{
  "id": "1732467123456",
  "content": "Your tweet content here...",
  "created_at": "2025-11-24T17:32:03.456Z",
  "updated_at": "2025-11-24T17:35:12.789Z"
}
```

## Status Indicators
The status bar shows:
- Character count
- 📎 Image attached (when image is present)
- 📝 Draft loaded (when editing an existing draft)

## Notes
- The TextArea widget handles all standard text editing operations
- **Emacs-style shortcuts** are used (Ctrl+A = start of line, Ctrl+E = end of line, etc.)
- **Text selection** works with Shift+arrow keys (tui-textarea limitation: no mouse click/drag)
- **Mouse wheel scrolling** is supported, but click-to-position is not (tui-textarea limitation)
- Character counter updates in real-time
- Clipboard image paste has been removed - use Ctrl+U to upload images from file path instead
- Instructions now wrap to two lines for better visibility

## Terminal Behavior Note
Most terminals allow you to bypass TUI apps and use native selection by holding **Shift** while clicking/dragging. This is a terminal feature, not part of the app itself.
