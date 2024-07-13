import type { ButtonProps } from "@widgets/button";
import Button from "@widgets/button";
import type { DownloadProgress } from "ky";
import { createSignal, Match, Show, splitProps, Switch } from "solid-js";
import { humanFileSize } from "../utils/size";
import { downloadFile } from "../api/file";
import Progress from "../widgets/progress";

export default function DownloadButton(
  props: ButtonProps & {
    icon: string;
    file: string;
    url: string;
    searchParams?: { [key: string]: string };
  }
) {
  const [downloadProps, btnProps] = splitProps(props, ["icon", "file", "url", "searchParams"]);
  const [downloading, setDownloading] = createSignal(false);
  const [downloadComplete, setDownloadComplete] = createSignal(false);
  const [progress, setProgress] = createSignal(null as DownloadProgress | null);

  function handleDownload() {
    setDownloading(true);
    void downloadFile(downloadProps.url, downloadProps.searchParams, (progress) => {
      setProgress(progress);
    })
      .then((blob) => {
        const url = window.URL.createObjectURL(blob as Blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = downloadProps.file;
        a.click();
        window.URL.revokeObjectURL(url);
      })
      .finally(() => {
        setDownloading(false);
        setDownloadComplete(true);
        setTimeout(() => {
          setDownloadComplete(false);
          setProgress(null);
        }, 1000);
      });
  }

  return (
    <Button
      {...btnProps}
      class={`relative ${btnProps.class}`.trim()}
      loading={downloading()}
      onClick={handleDownload}
      disabled={downloading()}
    >
      <Switch fallback={<span class={downloadProps.icon} />}>
        <Match when={downloading()}>{null}</Match>
        <Match when={downloadComplete()}>
          <span
            class={
              btnProps.size === "sm"
                ? "icon-[fluent--checkmark-16-regular] w-4 h-4 text-success"
                : "icon-[fluent--checkmark-20-regular] w-5 h-5 text-success"
            }
          />
        </Match>
      </Switch>
      <span>{downloadProps.file}</span>
      <Show when={downloading()}>
        <span class="mx-2 text-primary">
          {humanFileSize(progress()?.transferredBytes ?? 0, true)}/{humanFileSize(progress()?.totalBytes ?? 0, true)}
        </span>
        <Progress
          class="absolute left-1 right-1 bottom-0"
          min={0}
          max={progress()?.totalBytes ?? 1}
          value={progress()?.transferredBytes ?? 0}
          static
        />
      </Show>
    </Button>
  );
}
