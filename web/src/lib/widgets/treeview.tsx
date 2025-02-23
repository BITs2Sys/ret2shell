import { useSearchParams } from "@solidjs/router";
import clsx from "clsx";
import { For, type JSX, Show, createEffect, createSignal, untrack } from "solid-js";
import Button from "./button";
import Link from "./link";

export type TreeNode = {
  id: string | number;
  children: TreeNode[];
  name: string;
  icon: string;
  extraClasses?: string;
} & (
  | {
      type: "item";
      link?: string;
      period: [number | null, number | null];
      extraPart?: JSX.Element;
      searchValue?: string;
      onClick?: () => void;
    }
  | {
      type: "category";
    }
);

export type TreeViewProps = {
  tree: TreeNode[];
  size?: "sm" | "md";
  highlightPaths?: string[];
  activeMatch?: "exact" | "partial";
  activeSearchParams?: string;
};

function is_challenge_in_time(n: TreeNode) {
  return n.type === "item" && n.period[0] && n.period[1] && n.period[0] <= Date.now() && n.period[1] >= Date.now();
}

function parse_emoji(node: TreeNode) {
  if (node.type !== "category") return;
  if (/(Sign|Check)\s*In/i.test(node.name) || /签到/.test(node.name)) return "icon-[twemoji--wrapped-gift]";
  const week = node.name.match(/Week\s*(\d+)/i);
  if (node.children.find(is_challenge_in_time)) return "icon-[twemoji--sparkles]";
  if (week && !Number.isNaN(Number.parseInt(week[1]))) {
    const n = Number.parseInt(week[1]);
    if (n > 10) return "icon-[twemoji--keycap-pound]";
    return [
      "icon-[twemoji--keycap-0]",
      "icon-[twemoji--keycap-1]",
      "icon-[twemoji--keycap-2]",
      "icon-[twemoji--keycap-3]",
      "icon-[twemoji--keycap-4]",
      "icon-[twemoji--keycap-5]",
      "icon-[twemoji--keycap-6]",
      "icon-[twemoji--keycap-7]",
      "icon-[twemoji--keycap-8]",
      "icon-[twemoji--keycap-9]",
      "icon-[twemoji--keycap-10]",
    ][n];
  }
}

function resort_tree(tree: TreeNode[]) {
  const p1: TreeNode[] = [];
  const p2: TreeNode[] = [];
  const p3: TreeNode[] = [];
  while (tree.length > 0) {
    const node = tree.shift()!;
    if (node.children.find(is_challenge_in_time)) {
      if (/Week\s*(\d+)/i.test(node.name)) p2.push(node);
      else p1.push(node);
    } else p3.push(node);
  }
  tree.push(...p1, ...p2, ...p3);
  return tree;
}

export default function TreeView(props: TreeViewProps) {
  const [searchParams, _] = useSearchParams();
  const renderNode = (node: TreeNode, level = 0) => {
    const [showChildren, setShowChildren] = createSignal(false);
    createEffect(() => {
      if (props.highlightPaths) {
        untrack(() => {
          if (props.highlightPaths?.at(level) === node.id.toString()) {
            setShowChildren(true);
          }
        });
      }
    });
    return (
      <li>
        <Show
          when={node.type === "category"}
          fallback={
            <Link
              size={props.size}
              justify="start"
              ghost
              title={node.name}
              class={clsx("font-normal w-full overflow-hidden", node.extraClasses)}
              href={node.type === "item" && node.link ? node.link : "#"}
              activeMatch={props.activeMatch}
              active={
                node.type === "item" &&
                !!props.activeSearchParams &&
                searchParams[props.activeSearchParams] === node.searchValue
              }
            >
              <span class={clsx("w-5 h-5", node.icon)} />
              <span class="flex-1 text-start truncate">{node.name}</span>
              <Show when={node.type === "item" && node.extraPart}>{node.type === "item" && node.extraPart}</Show>
            </Link>
          }
        >
          <Button
            ghost
            title={node.name}
            size={props.size}
            justify="start"
            class={clsx("font-normal w-full overflow-hidden", node.extraClasses)}
            onClick={() => {
              setShowChildren(!showChildren());
            }}
          >
            <span class={clsx("w-5 h-5", parse_emoji(node) ?? node.icon)} />
            <span class="flex-1 text-start truncate">{node.name}</span>
            <span
              class={clsx(
                "icon-[fluent--chevron-right-20-regular] w-5 h-5 transition-transform",
                showChildren() && "rotate-90"
              )}
            />
          </Button>
        </Show>
        <Show when={node.type === "category" && showChildren()}>
          <ul class="mt-2 pl-2 relative before:absolute before:-top-2 before:bottom-0 before:left-2 before:w-[1px] before:bg-layer-content/20 flex flex-col space-y-2">
            <For each={node.children}>{(child) => renderNode(child, level + 1)}</For>
          </ul>
        </Show>
      </li>
    );
  };

  return (
    <ul class="flex flex-col space-y-2">
      <For each={resort_tree(props.tree)}>{(node) => renderNode(node)}</For>
    </ul>
  );
}
