import ace from "ace-builds";

type AceDefine = (
  id: string,
  deps: string[],
  factory: (
    require: (name: string) => unknown,
    exports: Record<string, unknown>,
    module: { exports: Record<string, unknown> }
  ) => void
) => void;

const aceModule = ace as typeof ace & { define: AceDefine };

aceModule.define(
  "ace/mode/rune-highlight-rules",
  ["require", "exports", "module", "ace/lib/oop", "ace/mode/text_highlight_rules"],
  (require, exports) => {
    const TextHighlightRules = (
      require("ace/mode/text_highlight_rules") as {
        TextHighlightRules: new () => { $rules: Record<string, unknown>; normalizeRules: () => void };
      }
    ).TextHighlightRules;
    const oop = require("ace/lib/oop") as { inherits: (child: unknown, parent: unknown) => void };

    const stringEscape = /\\(?:[nrt0'"\\]|x[\da-fA-F]{2}|u\{[\da-fA-F]{6}\})/.source;

    const RuneHighlightRules = function (this: { $rules: Record<string, unknown>; normalizeRules: () => void }) {
      // regexp must not have capturing parentheses. Use (?:) instead.
      // regexps are ordered -> the first match is used

      this.$rules = {
        start: [
          {
            token: "variable.other.source.rune",
            // `(?![\\\'])` to keep a lifetime name highlighting from continuing one character
            // past the name. The end `\'` will block this from matching for a character like
            // `'a'` (it should have character highlighting, not variable highlighting).
            regex: "'[a-zA-Z_][a-zA-Z0-9_]*(?![\\\\'])",
          },
          {
            token: "string.quoted.single.source.rune",
            regex: `'(?:[^'\\\\]|${stringEscape})'`,
          },
          {
            token: "identifier",
            regex: /r#[a-zA-Z_][a-zA-Z0-9_]*\b/,
          },
          {
            stateName: "bracketedComment",
            onMatch: function (
              this: { next?: string },
              value: string,
              currentState: string,
              stack: Array<string | number | undefined>
            ) {
              stack.unshift(this.next, value.length - 1, currentState);
              return "string.quoted.raw.source.rune";
            },
            regex: /r#*"/,
            next: [
              {
                onMatch: function (
                  this: { next?: string },
                  value: string,
                  _currentState: string,
                  stack: Array<string | number | undefined>
                ) {
                  let token = "string.quoted.raw.source.rune";
                  const stackLength = stack[1] as number | undefined;
                  if (stackLength !== undefined && value.length >= stackLength) {
                    if (value.length > stackLength) {
                      token = "invalid";
                    }
                    stack.shift();
                    stack.shift();
                    this.next = stack.shift() as string | undefined;
                  } else {
                    this.next = "";
                  }
                  return token;
                },
                regex: /"#*/,
                next: "start",
              },
              { defaultToken: "string.quoted.raw.source.rune" },
            ],
          },
          {
            token: "string.quoted.double.source.rune",
            regex: '"',
            push: [
              {
                token: "string.quoted.double.source.rune",
                regex: '"',
                next: "pop",
              },
              {
                token: "constant.character.escape.source.rune",
                regex: stringEscape,
              },
              { defaultToken: "string.quoted.double.source.rune" },
            ],
          },
          {
            token: "string.quoted.template.source.rune",
            regex: "`",
            push: [
              {
                token: "string.quoted.template.source.rune",
                regex: "`",
                next: "pop",
              },
              {
                token: "constant.character.escape.source.rune",
                regex: stringEscape,
              },
              { defaultToken: "string.quoted.template.source.rune" },
            ],
          },
          {
            token: ["keyword.source.rune", "text", "entity.name.function.source.rune"],
            regex: "\\b(fn)(\\s+)((?:r#)?[a-zA-Z_][a-zA-Z0-9_]*)",
          },
          { token: "support.constant", regex: "\\b[a-zA-Z_][\\w\\d]*::" },
          {
            token: "keyword.source.rune",
            regex:
              "\\b(?:abstract|alignof|as|async|await|become|box|break|catch|continue|const|crate|default|do|dyn|else|enum|extern|for|final|if|impl|in|let|loop|macro|match|mod|move|mut|offsetof|override|priv|proc|pub|pure|ref|return|self|sizeof|static|struct|super|trait|type|typeof|union|unsafe|unsized|use|virtual|where|while|yield)\\b",
          },
          {
            token: "storage.type.source.rune",
            regex: "\\b(?:Self|int|float|unit|char|bool|String|Bytes|GeneratorState|Generator|Future|Option|Result)\\b",
          },
          { token: "variable.language.source.rune", regex: "\\bself\\b" },
          {
            token: "comment.line.doc.source.rune",
            regex: "//!.*$",
          },
          {
            token: "comment.line.double-dash.source.rune",
            regex: "//.*$",
          },
          {
            token: "comment.start.block.source.rune",
            regex: "/\\*",
            stateName: "comment",
            push: [
              {
                token: "comment.start.block.source.rune",
                regex: "/\\*",
                push: "comment",
              },
              {
                token: "comment.end.block.source.rune",
                regex: "\\*/",
                next: "pop",
              },
              { defaultToken: "comment.block.source.rune" },
            ],
          },
          {
            token: "keyword.operator",
            // `[*/](?![*/])=?` is separated because `//` and `/* */` become comments and must be
            // guarded against. This states either `*` or `/` may be matched as long as the match
            // it isn't followed by either of the two. An `=` may be on the end.
            regex: /\$|[-=]>|[-+%^=!&|<>]=?|[*/](?![*/])=?/,
          },
          { token: "punctuation.operator", regex: /[?:,;.]/ },
          { token: "paren.lparen", regex: /[[({]/ },
          { token: "paren.rparen", regex: /[\])}]/ },
          {
            token: "constant.language.source.rune",
            regex: "\\b(?:true|false|Some|None|Ok|Err|Resume|Yield)\\b",
          },
          {
            token: "meta.preprocessor.source.rune",
            regex: "\\b\\w\\(\\w\\)*!|#\\[[\\w=\\(\\)_]+\\]\\b",
          },
          {
            token: "constant.numeric.source.rune",
            regex: /\b(?:0x[a-fA-F0-9_]+|0o[0-7_]+|0b[01_]+|[0-9][0-9_]*(?!\.))\b/,
          },
          {
            token: "constant.numeric.source.rune",
            regex: /\b(?:[0-9][0-9_]*)(?:\.[0-9][0-9_]*)?(?:[Ee][+-][0-9][0-9_]*)?\b/,
          },
        ],
      };

      this.normalizeRules();
    } as unknown as {
      new (): { $rules: Record<string, unknown>; normalizeRules: () => void };
    };

    const RuneHighlightRulesWithMeta = RuneHighlightRules as typeof RuneHighlightRules & {
      metaData?: {
        fileTypes: string[];
        foldingStartMarker: string;
        foldingStopMarker: string;
        name: string;
        scopeName: string;
      };
    };

    RuneHighlightRulesWithMeta.metaData = {
      fileTypes: ["rn"],
      foldingStartMarker: "^.*\\bfn\\s*(\\w+\\s*)?\\([^\\)]*\\)(\\s*\\{[^\\}]*)?\\s*$",
      foldingStopMarker: "^\\s*\\}",
      name: "Rust",
      scopeName: "source.rune",
    };

    oop.inherits(RuneHighlightRules, TextHighlightRules);

    exports.RuneHighlightRules = RuneHighlightRules;
  }
);

aceModule.define(
  "ace/mode/rune",
  ["require", "exports", "module", "ace/lib/oop", "ace/mode/text", "ace/mode/rune-highlight-rules"],
  (require, exports) => {
    const TextMode = (require("ace/mode/text") as { Mode: new () => unknown }).Mode;
    const RuneHighlightRules = (require("ace/mode/rune-highlight-rules") as { RuneHighlightRules: new () => unknown })
      .RuneHighlightRules;
    const oop = require("ace/lib/oop") as { inherits: (child: unknown, parent: unknown) => void };

    const Mode = function (this: {
      HighlightRules?: unknown;
      $behaviour?: unknown;
      $defaultBehaviour?: unknown;
    }) {
      this.HighlightRules = RuneHighlightRules;
      this.$behaviour = this.$defaultBehaviour;
    } as unknown as {
      new (): {
        HighlightRules?: unknown;
        $behaviour?: unknown;
        $defaultBehaviour?: unknown;
      };
      prototype: {
        lineCommentStart?: string;
        blockComment?: { start: string; end: string; nestable: boolean };
        $quotes?: Record<string, string>;
        $id?: string;
      };
    };

    oop.inherits(Mode, TextMode);

    (function (this: {
      lineCommentStart?: string;
      blockComment?: { start: string; end: string; nestable: boolean };
      $quotes?: Record<string, string>;
      $id?: string;
    }) {
      this.lineCommentStart = "//";
      this.blockComment = { start: "/*", end: "*/", nestable: true };
      this.$quotes = { '"': '"' };
      this.$id = "ace/mode/rune";
    }).call(Mode.prototype);

    exports.Mode = Mode;
  }
);
