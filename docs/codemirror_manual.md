# CodeMirror 5 User Manual

**VERSION 5.65.20**

CodeMirror is a code-editor component that can be embedded in Web pages. The core library provides only the editor component, no accompanying buttons, auto-completion, or other IDE functionality. It does provide a rich API on top of which such functionality can be straightforwardly implemented. See the addons included in the distribution, and 3rd party packages on npm, for reusable implementations of extra features.

CodeMirror works with language-specific modes. Modes are JavaScript programs that help color (and optionally indent) text written in a given language. The distribution comes with a number of modes (see the `mode/` directory), and it isn't hard to write new ones for other languages.

## Basic Usage

The easiest way to use CodeMirror is to simply load the script and style sheet found under `lib/` in the distribution, plus a mode script from one of the `mode/` directories. For example:

```html
<script src="lib/codemirror.js"></script>
<link rel="stylesheet" href="lib/codemirror.css">
<script src="mode/javascript/javascript.js"></script>
```

(Alternatively, use a module loader. More about that later.)

Having done this, an editor instance can be created like this:

```javascript
var myCodeMirror = CodeMirror(document.body);
```

The editor will be appended to the document body, will start empty, and will use the mode that we loaded. To have more control over the new editor, a configuration object can be passed to `CodeMirror` as a second argument:

```javascript
var myCodeMirror = CodeMirror(document.body, {
  value: "function myScript(){return 100;}\n",
  mode:  "javascript"
});
```

This will initialize the editor with a piece of code already in it, and explicitly tell it to use the JavaScript mode (which is useful when multiple modes are loaded). See below for a full discussion of the configuration options that CodeMirror accepts.

In cases where you don't want to append the editor to an element, and need more control over the way it is inserted, the first argument to the `CodeMirror` function can also be a function that, when given a DOM element, inserts it into the document somewhere. This could be used to, for example, replace a textarea with a real editor:

```javascript
var myCodeMirror = CodeMirror(function(elt) {
  myTextArea.parentNode.replaceChild(elt, myTextArea);
}, {value: myTextArea.value});
```

However, for this use case, which is a common way to use CodeMirror, the library provides a much more powerful shortcut:

```javascript
var myCodeMirror = CodeMirror.fromTextArea(myTextArea);
```

This will, among other things, ensure that the textarea's value is updated with the editor's contents when the form (if it is part of a form) is submitted. See the API reference for a full description of this method.

## Module Loaders

The files in the CodeMirror distribution contain shims for loading them (and their dependencies) in AMD or CommonJS environments. If the variables `exports` and `module` exist and have type `object`, CommonJS-style `require` will be used. If not, but there is a function `define` with an `amd` property present, AMD-style (RequireJS) will be used.

It is possible to use Browserify or similar tools to statically build modules using CodeMirror. Alternatively, use RequireJS to dynamically load dependencies at runtime. Both of these approaches have the advantage that they don't use the global namespace and can, thus, do things like load multiple versions of CodeMirror alongside each other.

Here's a simple example of using RequireJS to load CodeMirror:

```javascript
require([
  "cm/lib/codemirror", "cm/mode/htmlmixed/htmlmixed"
], function(CodeMirror) {
  CodeMirror.fromTextArea(document.getElementById("code"), {
    lineNumbers: true,
    mode: "htmlmixed"
  });
});
```

It will automatically load the modes that the mixed HTML mode depends on (XML, JavaScript, and CSS). Do not use RequireJS' `paths` option to configure the path to CodeMirror, since it will break loading submodules through relative paths. Use the `packages` configuration option instead, as in:

```javascript
require.config({
  packages: [{
    name: "codemirror",
    location: "../path/to/codemirror",
    main: "lib/codemirror"
  }]
});
```

## Configuration

Both the `CodeMirror` function and its `fromTextArea` method take as second (optional) argument an object containing configuration options. Any option not supplied like this will be taken from `CodeMirror.defaults`, an object containing the default options. You can update this object to change the defaults on your page.

Options are not checked in any way, so setting bogus option values is bound to lead to odd errors.

These are the supported options:

*   **`value`**: `string|CodeMirror.Doc`
    The starting value of the editor. Can be a string, or a document object.

*   **`mode`**: `string|object`
    The mode to use. When not given, this will default to the first mode that was loaded. It may be a string, which either simply names the mode or is a MIME type associated with the mode. The value `"null"` indicates no highlighting should be applied. Alternatively, it may be an object containing configuration options for the mode, with a `name` property that names the mode (for example `{name: "javascript", json: true}`). The demo pages for each mode contain information about what configuration parameters the mode supports. You can ask CodeMirror which modes and MIME types have been defined by inspecting the `CodeMirror.modes` and `CodeMirror.mimeModes` objects. The first maps mode names to their constructors, and the second maps MIME types to mode specs.

*   **`lineSeparator`**: `string|null`
    Explicitly set the line separator for the editor. By default (value `null`), the document will be split on CRLFs as well as lone CRs and LFs, and a single LF will be used as line separator in all output (such as `getValue`). When a specific string is given, lines will only be split on that string, and output will, by default, use that same separator.

*   **`theme`**: `string`
    The theme to style the editor with. You must make sure the CSS file defining the corresponding `.cm-s-[name]` styles is loaded (see the `theme` directory in the distribution). The default is `"default"`, for which colors are included in `codemirror.css`. It is possible to use multiple theming classes at once—for example `"foo bar"` will assign both the `cm-s-foo` and the `cm-s-bar` classes to the editor.

*   **`indentUnit`**: `integer`
    How many spaces a block (whatever that means in the edited language) should be indented. The default is 2.

*   **`smartIndent`**: `boolean`
    Whether to use the context-sensitive indentation that the mode provides (or just indent the same as the line before). Defaults to `true`.

*   **`tabSize`**: `integer`
    The width of a tab character. Defaults to 4.

*   **`indentWithTabs`**: `boolean`
    Whether, when indenting, the first N*tabSize spaces should be replaced by N tabs. Default is `false`.

*   **`electricChars`**: `boolean`
    Configures whether the editor should re-indent the current line when a character is typed that might change its proper indentation (only works if the mode supports indentation). Default is `true`.

*   **`specialChars`**: `RegExp`
    A regular expression used to determine which characters should be replaced by a special placeholder. Mostly useful for non-printing special characters. The default is `/[\\u0000-\u001f\u007f-\u009f\u00ad\u061c\u200b\u200e\u200f\u2028\u2029\u202d\u202e\u2066\u2067\u2069\ufeff\ufff9-\ufffc]/`.

*   **`specialCharPlaceholder`**: `function(char) → Element`
    A function that, given a special character identified by the `specialChars` option, produces a DOM node that is used to represent the character. By default, a red dot (•) is shown, with a `title` tooltip to indicate the character code.

*   **`direction`**: `"ltr" | "rtl"`
    Flips overall layout and selects base paragraph direction to be left-to-right or right-to-left. Default is `"ltr"`. CodeMirror applies the Unicode Bidirectional Algorithm to each line, but does not autodetect base direction — it's set to the editor direction for all lines. The resulting order is sometimes wrong when base direction doesn't match user intent (for example, leading and trailing punctuation jumps to the wrong side of the line). Therefore, it's helpful for multilingual input to let users toggle this option.

*   **`rtlMoveVisually`**: `boolean`
    Determines whether horizontal cursor movement through right-to-left (Arabic, Hebrew) text is visual (pressing the left arrow moves the cursor left) or logical (pressing the left arrow moves to the next lower index in the string, which is visually right in right-to-left text). The default is `false` on Windows, and `true` on other platforms.

*   **`keyMap`**: `string`
    Configures the key map to use. The default is `"default"`, which is the only key map defined in `codemirror.js` itself. Extra key maps are found in the `keymap` directory. See the section on key maps for more information.

*   **`extraKeys`**: `object`
    Can be used to specify extra key bindings for the editor, alongside the ones defined by `keyMap`. Should be either `null`, or a valid key map value.

*   **`configureMouse`**: `fn(cm: CodeMirror, repeat: "single" | "double" | "triple", event: Event) → Object`
    Allows you to configure the behavior of mouse selection and dragging. The function is called when the left mouse button is pressed. The returned object may have the following properties:
    *   **`unit`**: `"char" | "word" | "line" | "rectangle" | fn(CodeMirror, Pos) → {from: Pos, to: Pos}`
        The unit by which to select. May be one of the built-in units or a function that takes a position and returns a range around that, for a custom unit. The default is to return `"word"` for double clicks, `"line"` for triple clicks, `"rectangle"` for alt-clicks (or, on Chrome OS, meta-shift-clicks), and `"single"` otherwise.
    *   **`extend`**: `bool`
        Whether to extend the existing selection range or start a new one. By default, this is enabled when shift clicking.
    *   **`addNew`**: `bool`
        When enabled, this adds a new range to the existing selection, rather than replacing it. The default behavior is to enable this for command-click on Mac OS, and control-click on other platforms.
    *   **`moveOnDrag`**: `bool`
        When the mouse even drags content around inside the editor, this controls whether it is copied (`false`) or moved (`true`). By default, this is enabled by alt-clicking on Mac OS, and ctrl-clicking elsewhere.

*   **`lineWrapping`**: `boolean`
    Whether CodeMirror should scroll or wrap for long lines. Defaults to `false` (scroll).

*   **`lineNumbers`**: `boolean`
    Whether to show line numbers to the left of the editor.

*   **`firstLineNumber`**: `integer`
    At which number to start counting lines. Default is 1.

*   **`lineNumberFormatter`**: `function(line: integer) → string`
    A function used to format line numbers. The function is passed the line number, and should return a string that will be shown in the gutter.

*   **`gutters`**: `array<string | {className: string, style: ?string}>`
    Can be used to add extra gutters (beyond or instead of the line number gutter). Should be an array of CSS class names or class name / CSS string pairs, each of which defines a width (and optionally a background), and which will be used to draw the background of the gutters. May include the `CodeMirror-linenumbers` class, in order to explicitly set the position of the line number gutter (it will default to be to the right of all other gutters). These class names are the keys passed to `setGutterMarker`.

*   **`fixedGutter`**: `boolean`
    Determines whether the gutter scrolls along with the content horizontally (`false`) or whether it stays fixed during horizontal scrolling (`true`, the default).

*   **`scrollbarStyle`**: `string`
    Chooses a scrollbar implementation. The default is `"native"`, showing native scrollbars. The core library also provides the `"null"` style, which completely hides the scrollbars. Addons can implement additional scrollbar models.

*   **`coverGutterNextToScrollbar`**: `boolean`
    When `fixedGutter` is on, and there is a horizontal scrollbar, by default the gutter will be visible to the left of this scrollbar. If this option is set to `true`, it will be covered by an element with class `CodeMirror-gutter-filler`.

*   **`inputStyle`**: `string`
    Selects the way CodeMirror handles input and focus. The core library defines the `"textarea"` and `"contenteditable"` input models. On mobile browsers, the default is `"contenteditable"`. On desktop browsers, the default is `"textarea"`. Support for IME and screen readers is better in the `"contenteditable"` model. The intention is to make it the default on modern desktop browsers in the future.

*   **`readOnly`**: `boolean|string`
    This disables editing of the editor content by the user. If the special value `"nocursor"` is given (instead of simply `true`), focusing of the editor is also disallowed.

*   **`screenReaderLabel`**: `string`
    This label is read by the screenreaders when CodeMirror text area is focused. This is helpful for accessibility.

*   **`showCursorWhenSelecting`**: `boolean`
    Whether the cursor should be drawn when a selection is active. Defaults to `false`.

*   **`lineWiseCopyCut`**: `boolean`
    When enabled, which is the default, doing copy or cut when there is no selection will copy or cut the whole lines that have cursors on them.

*   **`pasteLinesPerSelection`**: `boolean`
    When pasting something from an external source (not from the editor itself), if the number of lines matches the number of selection, CodeMirror will by default insert one line per selection. You can set this to `false` to disable that behavior.

*   **`selectionsMayTouch`**: `boolean`
    Determines whether multiple selections are joined as soon as they touch (the default) or only when they overlap (`true`).

*   **`undoDepth`**: `integer`
    The maximum number of undo levels that the editor stores. Note that this includes selection change events. Defaults to 200.

*   **`historyEventDelay`**: `integer`
    The period of inactivity (in milliseconds) that will cause a new history event to be started when typing or deleting. Defaults to 1250.

*   **`tabindex`**: `integer`
    The tab index to assign to the editor. If not given, no tab index will be assigned.

*   **`autofocus`**: `boolean`
    Can be used to make CodeMirror focus itself on initialization. Defaults to off. When `fromTextArea` is used, and no explicit value is given for this option, it will be set to `true` when either the source textarea is focused, or it has an `autofocus` attribute and no other element is focused.

*   **`phrases`**: `?object`
    Some addons run user-visible strings (such as labels in the interface) through the `phrase` method to allow for translation. This option determines the return value of that method. When it is `null` or an object that doesn't have a property named by the input string, that string is returned. Otherwise, the value of the property corresponding to that string is returned.

Below this a few more specialized, low-level options are listed. These are only useful in very specific situations, you might want to skip them the first time you read this manual.

*   **`dragDrop`**: `boolean`
    Controls whether drag-and-drop is enabled. On by default.

*   **`allowDropFileTypes`**: `array<string>`
    When set (default is `null`) only files whose type is in the array can be dropped into the editor. The strings should be MIME types, and will be checked against the `type` of the `File` object as reported by the browser.

*   **`cursorBlinkRate`**: `number`
    Half-period in milliseconds used for cursor blinking. The default blink rate is 530ms. By setting this to zero, blinking can be disabled. A negative value hides the cursor entirely.

*   **`cursorScrollMargin`**: `number`
    How much extra space to always keep above and below the cursor when approaching the top or bottom of the visible view in a scrollable document. Default is 0.

*   **`cursorHeight`**: `number`
    Determines the height of the cursor. Default is 1, meaning it spans the whole height of the line. For some fonts (and by some tastes) a smaller height (for example 0.85), which causes the cursor to not reach all the way to the bottom of the line, looks better

*   **`singleCursorHeightPerLine`**: `boolean`
    If set to `true` (the default), will keep the cursor height constant for an entire line (or wrapped part of a line). When `false`, the cursor's height is based on the height of the adjacent reference character.

*   **`resetSelectionOnContextMenu`**: `boolean`
    Controls whether, when the context menu is opened with a click outside of the current selection, the cursor is moved to the point of the click. Defaults to `true`.

*   **`workTime`**, **`workDelay`**: `number`
    Highlighting is done by a pseudo background-thread that will work for `workTime` milliseconds, and then use timeout to sleep for `workDelay` milliseconds. The defaults are 200 and 300, you can change these options to make the highlighting more or less aggressive.

*   **`pollInterval`**: `number`
    Indicates how quickly CodeMirror should poll its input textarea for changes (when focused). Most input is captured by events, but some things, like IME input on some browsers, don't generate events that allow CodeMirror to properly detect it. Thus, it polls. Default is 100 milliseconds.

*   **`flattenSpans`**: `boolean`
    By default, CodeMirror will combine adjacent tokens into a single span if they have the same class. This will result in a simpler DOM tree, and thus perform better. With some kinds of styling (such as rounded corners), this will change the way the document looks. You can set this option to `false` to disable this behavior.

*   **`addModeClass`**: `boolean`
    When enabled (off by default), an extra CSS class will be added to each token, indicating the (inner) mode that produced it, prefixed with `"cm-m-"`. For example, tokens from the XML mode will get the `cm-m-xml` class.

*   **`maxHighlightLength`**: `number`
    When highlighting long lines, in order to stay responsive, the editor will give up and simply style the rest of the line as plain text when it reaches a certain position. The default is 10,000. You can set this to `Infinity` to turn off this behavior.

*   **`viewportMargin`**: `integer`
    Specifies the amount of lines that are rendered above and below the part of the document that's currently scrolled into view. This affects the amount of updates needed when scrolling, and the amount of work that such an update does. You should usually leave it at its default, 10. Can be set to `Infinity` to make sure the whole document is always rendered, and thus the browser's text search works on it. This will have bad effects on performance of big documents.

*   **`spellcheck`**: `boolean`
    Specifies whether or not spellcheck will be enabled on the input.

*   **`autocorrect`**: `boolean`
    Specifies whether or not autocorrect will be enabled on the input.

*   **`autocapitalize`**: `boolean`
    Specifies whether or not autocapitalization will be enabled on the input.

## Events

Various CodeMirror-related objects emit events, which allow client code to react to various situations. Handlers for such events can be registered with the `on` and `off` methods on the objects that the event fires on. To fire your own events, use `CodeMirror.signal(target, name, args...)`, where `target` is a non-DOM-node object.

An editor instance fires the following events. The `instance` argument always refers to the editor itself.

*   **`"change"`** `(instance: CodeMirror, changeObj: object)`
    Fires every time the content of the editor is changed. The `changeObj` is a `{from, to, text, removed, origin}` object containing information about the changes that occurred as second argument. `from` and `to` are the positions (in the pre-change coordinate system) where the change started and ended (for example, it might be `{ch:0, line:18}` if the position is at the beginning of line #19). `text` is an array of strings representing the text that replaced the changed range (split by line). `removed` is the text that used to be between `from` and `to`, which is overwritten by this change. This event is fired before the end of an operation, before the DOM updates happen.

*   **`"changes"`** `(instance: CodeMirror, changes: array<object>)`
    Like the `"change"` event, but batched per operation, passing an array containing all the changes that happened in the operation. This event is fired after the operation finished, and display changes it makes will trigger a new operation.

*   **`"beforeChange"`** `(instance: CodeMirror, changeObj: object)`
    This event is fired before a change is applied, and its handler may choose to modify or cancel the change. The `changeObj` object has `from`, `to`, and `text` properties, as with the `"change"` event. It also has a `cancel()` method, which can be called to cancel the change, and, if the change isn't coming from an undo or redo event, an `update(from, to, text)` method, which may be used to modify the change. Undo or redo changes can't be modified, because they hold some metainformation for restoring old marked ranges that is only valid for that specific change. All three arguments to `update` are optional, and can be left off to leave the existing value for that field intact. **Note**: you may not do anything from a `"beforeChange"` handler that would cause changes to the document or its visualization. Doing so will, since this handler is called directly from the bowels of the CodeMirror implementation, probably cause the editor to become corrupted.

*   **`"cursorActivity"`** `(instance: CodeMirror)`
    Will be fired when the cursor or selection moves, or any change is made to the editor content.

*   **`"keyHandled"`** `(instance: CodeMirror, name: string, event: Event)`
    Fired after a key is handled through a key map. `name` is the name of the handled key (for example `"Ctrl-X"` or `"'q'"`), and `event` is the DOM `keydown` or `keypress` event.

*   **`"inputRead"`** `(instance: CodeMirror, changeObj: object)`
    Fired whenever new input is read from the hidden textarea (typed or pasted by the user).

*   **`"electricInput"`** `(instance: CodeMirror, line: integer)`
    Fired if text input matched the mode's electric patterns, and this caused the line's indentation to change.

*   **`"beforeSelectionChange"`** `(instance: CodeMirror, obj: {ranges, origin, update})`
    This event is fired before the selection is moved. Its handler may inspect the set of selection ranges, present as an array of `{anchor, head}` objects in the `ranges` property of the `obj` argument, and optionally change them by calling the `update` method on this object, passing an array of ranges in the same format. The object also contains an `origin` property holding the origin string passed to the selection-changing method, if any. Handlers for this event have the same restriction as `"beforeChange"` handlers — they should not do anything to directly update the state of the editor.

*   **`"viewportChange"`** `(instance: CodeMirror, from: number, to: number)`
    Fires whenever the view port of the editor changes (due to scrolling, editing, or any other factor). The `from` and `to` arguments give the new start and end of the viewport.

*   **`"swapDoc"`** `(instance: CodeMirror, oldDoc: Doc)`
    This is signalled when the editor's document is replaced using the `swapDoc` method.

*   **`"gutterClick"`** `(instance: CodeMirror, line: integer, gutter: string, clickEvent: Event)`
    Fires when the editor gutter (the line-number area) is clicked. Will pass the editor instance as first argument, the (zero-based) number of the line that was clicked as second argument, the CSS class of the gutter that was clicked as third argument, and the raw `mousedown` event object as fourth argument.

*   **`"gutterContextMenu"`** `(instance: CodeMirror, line: integer, gutter: string, contextMenu: Event: Event)`
    Fires when the editor gutter (the line-number area) receives a `contextmenu` event. Will pass the editor instance as first argument, the (zero-based) number of the line that was clicked as second argument, the CSS class of the gutter that was clicked as third argument, and the raw `contextmenu` mouse event object as fourth argument. You can `preventDefault` the event, to signal that CodeMirror should do no further handling.

*   **`"focus"`** `(instance: CodeMirror, event: Event)`
    Fires whenever the editor is focused.

*   **`"blur"`** `(instance: CodeMirror, event: Event)`
    Fires whenever the editor is unfocused.

*   **`"scroll"`** `(instance: CodeMirror)`
    Fires when the editor is scrolled.

*   **`"refresh"`** `(instance: CodeMirror)`
    Fires when the editor is refreshed or resized. Mostly useful to invalidate cached values that depend on the editor or character size. See also the `autorefresh` addon.

*   **`"optionChange"`** `(instance: CodeMirror, option: string)`
    Dispatched every time an option is changed with `setOption`.

*   **`"scrollCursorIntoView"`** `(instance: CodeMirror, event: Event)`
    Fires when the editor tries to scroll its cursor into view. Can be hooked into to take care of additional scrollable containers around the editor. When the event object has its `preventDefault` method called, CodeMirror will not itself try to scroll the window.

*   **`"update"`** `(instance: CodeMirror)`
    Will be fired whenever CodeMirror updates its DOM display.

*   **`"renderLine"`** `(instance: CodeMirror, line: LineHandle, element: Element)`
    Fired whenever a line is (re-)rendered to the DOM. Fired right after the DOM element is built, before it is added to the document. The handler may mess with the style of the resulting element, or add event handlers, but should not try to change the state of the editor.

*   **`"mousedown"`, `"dblclick"`, `"touchstart"`, `"contextmenu"`, `"keydown"`, `"keypress"`, `"keyup"`, `"cut"`, `"copy"`, `"paste"`, `"dragstart"`, `"dragenter"`, `"dragover"`, `"dragleave"`, `"drop"`** `(instance: CodeMirror, event: Event)`
    Fired when CodeMirror is handling a DOM event of this type. You can `preventDefault` the event, or give it a truthy `codemirrorIgnore` property, to signal that CodeMirror should do no further handling.

Document objects (instances of `CodeMirror.Doc`) emit the following events:

*   **`"change"`** `(doc: CodeMirror.Doc, changeObj: object)`
    Fired whenever a change occurs to the document. `changeObj` has a similar type as the object passed to the editor's `"change"` event.

*   **`"beforeChange"`** `(doc: CodeMirror.Doc, change: object)`
    See the description of the same event on editor instances.

*   **`"cursorActivity"`** `(doc: CodeMirror.Doc)`
    Fired whenever the cursor or selection in this document changes.

*   **`"beforeSelectionChange"`** `(doc: CodeMirror.Doc, selection: {head, anchor})`
    Equivalent to the event by the same name as fired on editor instances.

Line handles (as returned by, for example, `getLineHandle`) support these events:

*   **`"delete"`** `()`
    Will be fired when the line object is deleted. A line object is associated with the start of the line. Mostly useful when you need to find out when your gutter markers on a given line are removed.

*   **`"change"`** `(line: LineHandle, changeObj: object)`
    Fires when the line's text content is changed in any way (but the line is not deleted outright). The change object is similar to the one passed to `change` event on the editor object.

Marked range handles (`CodeMirror.TextMarker`), as returned by `markText` and `setBookmark`, emit the following events:

*   **`"beforeCursorEnter"`** `()`
    Fired when the cursor enters the marked range. From this event handler, the editor state may be inspected but not modified, with the exception that the range on which the event fires may be cleared.

*   **`"clear"`** `(from: {line, ch}, to: {line, ch})`
    Fired when the range is cleared, either through cursor movement in combination with `clearOnEnter` or through a call to its `clear()` method. Will only be fired once per handle. Note that deleting the range through text editing does not fire this event, because an undo action might bring the range back into existence. `from` and `to` give the part of the document that the range spanned when it was cleared.

*   **`"hide"`** `()`
    Fired when the last part of the marker is removed from the document by editing operations.

*   **`"unhide"`** `()`
    Fired when, after the marker was removed by editing, a undo operation brought the marker back.

Line widgets (`CodeMirror.LineWidget`), returned by `addLineWidget`, fire these events:

*   **`"redraw"`** `()`
    Fired whenever the editor re-adds the widget to the DOM. This will happen once right after the widget is added (if it is scrolled into view), and then again whenever it is scrolled out of view and back in again, or when changes to the editor options or the line the widget is on require the widget to be redrawn.

## Key Maps

Key maps are ways to associate keys and mouse buttons with functionality. A key map is an object mapping strings that identify the buttons to functions that implement their functionality.

The CodeMirror distributions comes with Emacs, Vim, and Sublime Text-style keymaps.

Keys are identified either by name or by character. The `CodeMirror.keyNames` object defines names for common keys and associates them with their key codes. Examples of names defined here are `Enter`, `F5`, and `Q`. These can be prefixed with `Shift-`, `Cmd-`, `Ctrl-`, and `Alt-` to specify a modifier. So for example, `Shift-Ctrl-Space` would be a valid key identifier.

Common example: map the `Tab` key to insert spaces instead of a tab character.

```javascript
editor.setOption("extraKeys", {
  Tab: function(cm) {
    var spaces = Array(cm.getOption("indentUnit") + 1).join(" ");
    cm.replaceSelection(spaces);
  }
});
```

Alternatively, a character can be specified directly by surrounding it in single quotes, for example `'$'` or `'q'`. Due to limitations in the way browsers fire key events, these may not be prefixed with modifiers.

To bind mouse buttons, use the names `LeftClick`, `MiddleClick`, and `RightClick`. These can also be prefixed with modifiers, and in addition, the word `Double` or `Triple` can be put before `Click` (as in `LeftDoubleClick`) to bind a double- or triple-click. The function for such a binding is passed the position that was clicked as second argument.

Multi-stroke key bindings can be specified by separating the key names by spaces in the property name, for example `Ctrl-X Ctrl-V`. When a map contains multi-stoke bindings or keys with modifiers that are not specified in the default order (`Shift-Cmd-Ctrl-Alt`), you must call `CodeMirror.normalizeKeyMap` on it before it can be used. This function takes a keymap and modifies it to normalize modifier order and properly recognize multi-stroke bindings. It will return the keymap itself.

The `CodeMirror.keyMap` object associates key maps with names. User code and key map definitions can assign extra properties to this object. Anywhere where a key map is expected, a string can be given, which will be looked up in this object. It also contains the `"default"` key map holding the default bindings.

The values of properties in key maps can be either functions of a single argument (the CodeMirror instance), strings, or `false`. Strings refer to commands, which are described below. If the property is set to `false`, CodeMirror leaves handling of the key up to the browser. A key handler function may return `CodeMirror.Pass` to indicate that it has decided not to handle the key, and other handlers (or the default behavior) should be given a turn.

Keys mapped to command names that start with the characters `"go"` or to functions that have a truthy `motion` property (which should be used for cursor-movement actions) will be fired even when an extra `Shift` modifier is present (i.e. `"Up": "goLineUp"` matches both up and shift-up). This is used to easily implement shift-selection.

Key maps can defer to each other by defining a `fallthrough` property. This indicates that when a key is not found in the map itself, one or more other maps should be searched. It can hold either a single key map or an array of key maps.

When a key map needs to set something up when it becomes active, or tear something down when deactivated, it can contain `attach` and/or `detach` properties, which should hold functions that take the editor instance and the next or previous keymap. Note that this only works for the top-level keymap, not for `fallthrough` maps or maps added with `extraKeys` or `addKeyMap`.

## Commands

Commands are parameter-less actions that can be performed on an editor. Their main use is for key bindings. Commands are defined by adding properties to the `CodeMirror.commands` object. A number of common commands are defined by the library itself, most of them used by the default key bindings. The value of a command property must be a function of one argument (an editor instance).

Some of the commands below are referenced in the default key map, but not defined by the core library. These are intended to be defined by user code or addons.

Commands can also be run with the `execCommand` method.

*   **`selectAll`** `Ctrl-A` (PC), `Cmd-A` (Mac)
    Select the whole content of the editor.

*   **`singleSelection`** `Esc`
    When multiple selections are present, this deselects all but the primary selection.

*   **`killLine`** `Ctrl-K` (Mac)
    Emacs-style line killing. Deletes the part of the line after the cursor. If that consists only of whitespace, the newline at the end of the line is also deleted.

*   **`deleteLine`** `Ctrl-D` (PC), `Cmd-D` (Mac)
    Deletes the whole line under the cursor, including newline at the end.

*   **`delLineLeft`**
    Delete the part of the line before the cursor.

*   **`delWrappedLineLeft`** `Cmd-Backspace` (Mac)
    Delete the part of the line from the left side of the visual line the cursor is on to the cursor.

*   **`delWrappedLineRight`** `Cmd-Delete` (Mac)
    Delete the part of the line from the cursor to the right side of the visual line the cursor is on.

*   **`undo`** `Ctrl-Z` (PC), `Cmd-Z` (Mac)
    Undo the last change. Note that, because browsers still don't make it possible for scripts to react to or customize the context menu, selecting undo (or redo) from the context menu in a CodeMirror instance does not work.

*   **`redo`** `Ctrl-Y` (PC), `Shift-Cmd-Z` (Mac), `Cmd-Y` (Mac)
    Redo the last undone change.

*   **`undoSelection`** `Ctrl-U` (PC), `Cmd-U` (Mac)
    Undo the last change to the selection, or if there are no selection-only changes at the top of the history, undo the last change.

*   **`redoSelection`** `Alt-U` (PC), `Shift-Cmd-U` (Mac)
    Redo the last change to the selection, or the last text change if no selection changes remain.

*   **`goDocStart`** `Ctrl-Home` (PC), `Cmd-Up` (Mac), `Cmd-Home` (Mac)
    Move the cursor to the start of the document.

*   **`goDocEnd`** `Ctrl-End` (PC), `Cmd-End` (Mac), `Cmd-Down` (Mac)
    Move the cursor to the end of the document.

*   **`goLineStart`** `Alt-Left` (PC), `Ctrl-A` (Mac)
    Move the cursor to the start of the line.

*   **`goLineStartSmart`** `Home`
    Move to the start of the text on the line, or if we are already there, to the actual start of the line (including whitespace).

*   **`goLineEnd`** `Alt-Right` (PC), `Ctrl-E` (Mac)
    Move the cursor to the end of the line.

*   **`goLineRight`** `Cmd-Right` (Mac)
    Move the cursor to the right side of the visual line it is on.

*   **`goLineLeft`** `Cmd-Left` (Mac)
    Move the cursor to the left side of the visual line it is on. If this line is wrapped, that may not be the start of the line.

*   **`goLineLeftSmart`**
    Move the cursor to the left side of the visual line it is on. If that takes it to the start of the line, behave like `goLineStartSmart`.

*   **`goLineUp`** `Up`, `Ctrl-P` (Mac)
    Move the cursor up one line.

*   **`goLineDown`** `Down`, `Ctrl-N` (Mac)
    Move down one line.

*   **`goPageUp`** `PageUp`, `Shift-Ctrl-V` (Mac)
    Move the cursor up one screen, and scroll up by the same distance.

*   **`goPageDown`** `PageDown`, `Ctrl-V` (Mac)
    Move the cursor down one screen, and scroll down by the same distance.

*   **`goCharLeft`** `Left`, `Ctrl-B` (Mac)
    Move the cursor one character left, going to the previous line when hitting the start of line.

*   **`goCharRight`** `Right`, `Ctrl-F` (Mac)
    Move the cursor one character right, going to the next line when hitting the end of line.

*   **`goColumnLeft`**
    Move the cursor one character left, but don't cross line boundaries.

*   **`goColumnRight`**
    Move the cursor one character right, don't cross line boundaries.

*   **`goWordLeft`** `Alt-B` (Mac)
    Move the cursor to the start of the previous word.

*   **`goWordRight`** `Alt-F` (Mac)
    Move the cursor to the end of the next word.

*   **`goGroupLeft`** `Ctrl-Left` (PC), `Alt-Left` (Mac)
    Move to the left of the group before the cursor. A group is a stretch of word characters, a stretch of punctuation characters, a newline, or a stretch of more than one whitespace character.

*   **`goGroupRight`** `Ctrl-Right` (PC), `Alt-Right` (Mac)
    Move to the right of the group after the cursor (see above).

*   **`delCharBefore`** `Shift-Backspace`, `Ctrl-H` (Mac)
    Delete the character before the cursor.

*   **`delCharAfter`** `Delete`, `Ctrl-D` (Mac)
    Delete the character after the cursor.

*   **`delWordBefore`** `Alt-Backspace` (Mac)
    Delete up to the start of the word before the cursor.

*   **`delWordAfter`** `Alt-D` (Mac)
    Delete up to the end of the word after the cursor.

*   **`delGroupBefore`** `Ctrl-Backspace` (PC), `Alt-Backspace` (Mac)
    Delete to the left of the group before the cursor.

*   **`delGroupAfter`** `Ctrl-Delete` (PC), `Ctrl-Alt-Backspace` (Mac), `Alt-Delete` (Mac)
    Delete to the start of the group after the cursor.

*   **`indentAuto`** `Shift-Tab`
    Auto-indent the current line or selection.

*   **`indentMore`** `Ctrl-]` (PC), `Cmd-]` (Mac)
    Indent the current line or selection by one indent unit.

*   **`indentLess`** `Ctrl-[` (PC), `Cmd-[` (Mac)
    Dedent the current line or selection by one indent unit.

*   **`insertTab`**
    Insert a tab character at the cursor.

*   **`insertSoftTab`**
    Insert the amount of spaces that match the width a tab at the cursor position would have.

*   **`defaultTab`** `Tab`
    If something is selected, indent it by one indent unit. If nothing is selected, insert a tab character.

*   **`transposeChars`** `Ctrl-T` (Mac)
    Swap the characters before and after the cursor.

*   **`newlineAndIndent`** `Enter`
    Insert a newline and auto-indent the new line.

*   **`toggleOverwrite`** `Insert`
    Flip the overwrite flag.

*   **`save`** `Ctrl-S` (PC), `Cmd-S` (Mac)
    Not defined by the core library, only referred to in key maps. Intended to provide an easy way for user code to define a save command.

*   **`find`** `Ctrl-F` (PC), `Cmd-F` (Mac)
*   **`findNext`** `Ctrl-G` (PC), `Cmd-G` (Mac)
*   **`findPrev`** `Shift-Ctrl-G` (PC), `Shift-Cmd-G` (Mac)
*   **`replace`** `Shift-Ctrl-F` (PC), `Cmd-Alt-F` (Mac)
*   **`replaceAll`** `Shift-Ctrl-R` (PC), `Shift-Cmd-Alt-F` (Mac)
    Not defined by the core library, but defined in the search addon (or custom client addons).

## Customized Styling

Up to a certain extent, CodeMirror's look can be changed by modifying style sheet files. The style sheets supplied by modes simply provide the colors for that mode, and can be adapted in a very straightforward way. To style the editor itself, it is possible to alter or override the styles defined in `codemirror.css`. 

Some care must be taken there, since a lot of the rules in this file are necessary to have CodeMirror function properly. Adjusting colors should be safe, of course, and with some care a lot of other things can be changed as well. The CSS classes defined in this file serve the following roles:

*   **`CodeMirror`**
    The outer element of the editor. This should be used for the editor width, height, borders and positioning. Can also be used to set styles that should hold for everything inside the editor (such as font and font size), or to set a background. Setting this class' height style to `auto` will make the editor resize to fit its content (it is recommended to also set the `viewportMargin` option to `Infinity` when doing this.

*   **`CodeMirror-focused`**
    Whenever the editor is focused, the top element gets this class. This is used to hide the cursor and give the selection a different color when the editor is not focused.

*   **`CodeMirror-gutters`**
    This is the backdrop for all gutters. Use it to set the default gutter background color, and optionally add a border on the right of the gutters.

*   **`CodeMirror-linenumbers`**
    Use this for giving a background or width to the line number gutter.

*   **`CodeMirror-linenumber`**
    Used to style the actual individual line numbers. These won't be children of the `CodeMirror-linenumbers` (plural) element, but rather will be absolutely positioned to overlay it. Use this to set alignment and text properties for the line numbers.

*   **`CodeMirror-lines`**
    The visible lines. This is where you specify vertical padding for the editor content.

*   **`CodeMirror-cursor`**
    The cursor is a block element that is absolutely positioned. You can make it look whichever way you want.

*   **`CodeMirror-selected`**
    The selection is represented by `span` elements with this class.

*   **`CodeMirror-matchingbracket`**, **`CodeMirror-nonmatchingbracket`**
    These are used to style matched (or unmatched) brackets.

If your page's style sheets do funky things to all `div` or `pre` elements (you probably shouldn't do that), you'll have to define rules to cancel these effects out again for elements under the `CodeMirror` class.

Themes are also simply CSS files, which define colors for various syntactic elements. See the files in the `theme` directory.

## Programming API

A lot of CodeMirror features are only available through its API. Thus, you need to write code (or use addons) if you want to expose them to your users.

Whenever points in the document are represented, the API uses objects with `line` and `ch` properties. Both are zero-based. CodeMirror makes sure to 'clip' any positions passed by client code so that they fit inside the document, so you shouldn't worry too much about sanitizing your coordinates. If you give `ch` a value of `null`, or don't specify it, it will be replaced with the length of the specified line. Such positions may also have a `sticky` property holding `"before"` or `"after"`, whether the position is associated with the character before or after it. This influences, for example, where the cursor is drawn on a line-break or bidi-direction boundary.

Methods prefixed with `doc.` can, unless otherwise specified, be called both on `CodeMirror` (editor) instances and `CodeMirror.Doc` instances. Methods prefixed with `cm.` are only available on `CodeMirror` instances.

### Constructor

Constructing an editor instance is done with the `CodeMirror(place: Element|fn(Element), ?option: object)` constructor. If the `place` argument is a DOM element, the editor will be appended to it. If it is a function, it will be called, and is expected to place the editor into the document. `options` may be an element mapping option names to values. The options that it doesn't explicitly specify (or all options, if it is not passed) will be taken from `CodeMirror.defaults`.

Note that the `options` object passed to the constructor will be mutated when the instance's options are changed, so you shouldn't share such objects between instances.

See `CodeMirror.fromTextArea` for another way to construct an editor instance.

### Content Manipulation Methods

*   **`doc.getValue(?separator: string) → string`**
    Get the current editor content. You can pass it an optional argument to specify the string to be used to separate lines (defaults to `\n`).

*   **`doc.setValue(content: string)`**
    Set the editor content.

*   **`doc.getRange(from: {line, ch}, to: {line, ch}, ?separator: string) → string`**
    Get the text between the given points in the editor, which should be `{line, ch}` objects. An optional third argument can be given to indicate the line separator string to use (defaults to `\n`).

*   **`doc.replaceRange(replacement: string, from: {line, ch}, to: {line, ch}, ?origin: string)`**
    Replace the part of the document between `from` and `to` with the given string. `from` and `to` must be `{line, ch}` objects. `to` can be left off to simply insert the string at position `from`. When `origin` is given, it will be passed on to `"change"` events, and its first letter will be used to determine whether this change can be merged with previous history events, in the way described for selection origins.

*   **`doc.getLine(n: integer) → string`**
    Get the content of line `n`.

*   **`doc.lineCount() → integer`**
    Get the number of lines in the editor.

*   **`doc.firstLine() → integer`**
    Get the number of first line in the editor. This will usually be zero but for linked sub-views, or documents instantiated with a non-zero first line, it might return other values.

*   **`doc.lastLine() → integer`**
    Get the number of last line in the editor. This will usually be `doc.lineCount() - 1`, but for linked sub-views, it might return other values.

*   **`doc.getLineHandle(num: integer) → LineHandle`**
    Fetches the line handle for the given line number.

*   **`doc.getLineNumber(handle: LineHandle) → integer`**
    Given a line handle, returns the current position of that line (or `null` when it is no longer in the document).

*   **`doc.eachLine(f: (line: LineHandle))`**
*   **`doc.eachLine(start: integer, end: integer, f: (line: LineHandle))`**
    Iterate over the whole document, or if `start` and `end` line numbers are given, the range from `start` up to (not including) `end`, and call `f` for each line, passing the line handle. `eachLine` stops iterating if `f` returns truthy value. This is a faster way to visit a range of line handlers than calling `getLineHandle` for each of them. Note that line handles have a `text` property containing the line's content (as a string).

*   **`doc.markClean()`**
    Set the editor content as 'clean', a flag that it will retain until it is edited, and which will be set again when such an edit is undone again. Useful to track whether the content needs to be saved. This function is deprecated in favor of `changeGeneration`, which allows multiple subsystems to track different notions of cleanness without interfering.

*   **`doc.changeGeneration(?closeEvent: boolean) → integer`**
    Returns a number that can later be passed to `isClean` to test whether any edits were made (and not undone) in the meantime. If `closeEvent` is true, the current history event will be ‘closed’, meaning it can't be combined with further changes (rapid typing or deleting events are typically combined).

*   **`doc.isClean(?generation: integer) → boolean`**
    Returns whether the document is currently clean — not modified since initialization or the last call to `markClean` if no argument is passed, or since the matching call to `changeGeneration` if a generation value is given.

### Cursor and Selection Methods

*   **`doc.getSelection(?lineSep: string) → string`**
    Get the currently selected code. Optionally pass a line separator to put between the lines in the output. When multiple selections are present, they are concatenated with instances of `lineSep` in between.

*   **`doc.getSelections(?lineSep: string) → array<string>`**
    Returns an array containing a string for each selection, representing the content of the selections.

*   **`doc.replaceSelection(replacement: string, ?select: string)`**
    Replace the selection(s) with the given string. By default, the new selection ends up after the inserted text. The optional `select` argument can be used to change this—passing `