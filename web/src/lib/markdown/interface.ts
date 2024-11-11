export type MarkToType = "html" | "terminal";

export interface MarkToHtmlOptions {
  math?: boolean;
  code?: boolean;
  headingAnchors?: boolean;
  toc?: boolean;
}
