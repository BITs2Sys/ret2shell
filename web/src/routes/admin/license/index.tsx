import { usePlatformLicense } from "@api/platform";
import Spin from "@assets/animates/spin";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Article from "@widgets/article";
import Divider from "@widgets/divider";
import { createMemo, Show } from "solid-js";

function escapeInlineMarkdown(content: string) {
  return content.replaceAll("`", "\\`");
}

function toCodeBlock(content: string) {
  return ["```text", content, "```"].join("\n");
}

function formatLicenseMarkdown(content: string) {
  if (!content.trim()) return "";

  const sections = content
    .replaceAll("\r\n", "\n")
    .trim()
    .split(/\n\s*\n/g);
  const blocks: string[] = [];
  let inApplication = false;

  for (const section of sections) {
    const lines = section.split("\n");
    const trimmedLines = lines.map((line) => line.trim()).filter((line) => line.length > 0);

    if (trimmedLines.length === 0) continue;

    const firstLine = trimmedLines[0];
    const joined = escapeInlineMarkdown(trimmedLines.join(" "));
    const isIndented = lines.every((line) => line.length === 0 || /^\s{4}/.test(line));

    if (firstLine === "GNU GENERAL PUBLIC LICENSE") {
      blocks.push("# GNU General Public License");
      if (trimmedLines[1]) blocks.push(`_${escapeInlineMarkdown(trimmedLines[1])}_`);
      continue;
    }

    if (firstLine === "RET2SHELL PUBLIC LICENSE") {
      blocks.push("# Ret2Shell Public License");
      if (trimmedLines[1]) blocks.push(`_${escapeInlineMarkdown(trimmedLines[1])}_`);
      continue;
    }

    if (firstLine === "Preamble") {
      blocks.push("## Preamble");
      continue;
    }

    if (firstLine === "APPENDIX A") {
      blocks.push("## Appendix A");
      continue;
    }

    if (firstLine === "TERMS AND CONDITIONS") {
      blocks.push("## Terms and Conditions");
      continue;
    }

    if (firstLine === "END OF TERMS AND CONDITIONS") {
      blocks.push("## End of Terms and Conditions");
      continue;
    }

    if (firstLine === "How to Apply These Terms to Your New Programs") {
      inApplication = true;
      blocks.push("## How to Apply These Terms to Your New Programs");
      continue;
    }

    if (/^\d+\.\s+/.test(firstLine)) {
      blocks.push(`### ${escapeInlineMarkdown(firstLine)}`);
      continue;
    }

    if (/^[a-z]\)\s+/.test(firstLine)) {
      blocks.push(`- ${joined}`);
      continue;
    }

    if (
      isIndented &&
      inApplication &&
      (firstLine.startsWith("<") ||
        firstLine.startsWith("This program") ||
        firstLine.startsWith("You should have received") ||
        firstLine.startsWith("<program>") ||
        firstLine.startsWith("This is free software"))
    ) {
      blocks.push(toCodeBlock(lines.map((line) => line.replace(/^ {4}/, "")).join("\n")));
      continue;
    }

    blocks.push(joined);
  }

  return blocks.join("\n\n");
}

export default function License() {
  const license = usePlatformLicense({ enabled: () => true });

  const licenseData = createMemo(() => license.data);
  const licenseContent = createMemo(() => licenseData()?.content || "");
  const licenseMarkdown = createMemo(() => formatLicenseMarkdown(licenseContent()));

  return (
    <>
      <Title page={licenseData()?.name || t("license.title")} route="/license" />
      <div class="flex-1 flex flex-col items-center px-3 lg:px-6">
        <h1 class="text-3xl flex flex-row space-x-4 items-center w-full max-w-5xl justify-start print:justify-center font-bold mt-8 print:mt-16">
          <Show
            when={licenseData()?.name}
            fallback={
              <>
                <Spin width={32} height={32} />
                <span>{t("general.loading.short")}</span>
              </>
            }
          >
            <span>{licenseData()?.name}</span>
          </Show>
        </h1>
        <div class="flex flex-row items-center w-full max-w-5xl justify-start print:justify-center space-x-6 print:space-x-2 opacity-60 flex-wrap py-3">
          <a
            class="hover:underline font-bold flex flex-row space-x-2 items-center print:hidden"
            href={licenseData()?.url || "/license"}
            rel="noreferrer"
            target="_blank"
          >
            <span class="shrink-0 icon-[fluent--open-20-regular] w-5 h-5" />
            <span>{licenseData()?.spdx_id || "LicenseRef-Ret2Shell-Public-2.0"}</span>
          </a>
          <button
            class="cursor-pointer font-bold hover:underline flex flex-row space-x-2 items-center print:hidden"
            onClick={() => print()}
            type="button"
          >
            <span class="shrink-0 icon-[fluent--print-20-regular] w-5 h-5" />
            <span>{t("general.actions.print.title")}</span>
          </button>
        </div>
        <Divider class="w-full max-w-5xl" />
        <Show
          when={licenseContent()}
          fallback={
            <div class="w-full max-w-5xl flex justify-center py-16">
              <Spin width={32} height={32} />
            </div>
          }
        >
          <Article class="self-center" content={licenseMarkdown()} extra headingAnchors toc />
        </Show>
      </div>
    </>
  );
}
