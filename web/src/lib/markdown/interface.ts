export type MarkToType = "html" | "terminal";

export interface MarkToHtmlOptions {
  alertBlockquote?: boolean;
  math?: boolean;
  code?: boolean;
  headingAnchors?: boolean;
  toc?: boolean;
}
