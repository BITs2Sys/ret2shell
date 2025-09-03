import { api_root, handleHttpError } from "@api";
import { getPlatformLogs } from "@api/platform";
import DownloadButton from "@blocks/download-button";
import { createBreakpoints } from "@solid-primitives/media";
import { A, useSearchParams } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { Title } from "@storage/header";
import { breakpoints, t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Button from "@widgets/button";
import LoadingTips from "@widgets/loading-tips";
import Tag from "@widgets/tag";
import clsx from "clsx";
import { DateTime } from "luxon";
import { createSignal, For, Match, onCleanup, onMount, Show, Switch } from "solid-js";

type LogSpan = {
  name: string;
  [key: string]: unknown;
};

type Log = {
  timestamp: string;
  level: string;
  target: string;
  fields: Record<string, unknown>;
  span?: LogSpan;
  spans?: LogSpan[];
};

function SpanHttp(log: Log) {
  if (!log.spans) return null;
  for (const span of log.spans) {
    if (span.name === "http") {
      return span;
    }
  }
}

function SpanUser(log: Log) {
  if (!log.spans) return null;
  for (const span of log.spans) {
    if (span.name === "user") {
      return (
        <A class="hover:underline" target="_blank" rel="noreferrer" href={`/admin/users?user=${span.id}`}>
          {span.account as string}
        </A>
      );
    }
  }
}

function LogField(log: Log) {
  if (!log.fields) return null;
  return (
    <For each={Object.entries(log.fields)}>
      {([key, value]) => (
        <>
          <span class="italic opacity-60">{key}</span>
          <span class="opacity-60">=</span>
          <span>{value as string}&nbsp;</span>
        </>
      )}
    </For>
  );
}

export default function () {
  const [loading, setLoading] = createSignal(false);
  const [enableStreamLogs, setEnableStreamLogs] = createSignal(false);
  const [logs, setLogs] = createSignal([] as Log[]);
  const [searchParams, setSearchParams] = useSearchParams();
  let ws: WebSocket;

  const [logFiles, setLogFiles] = createSignal([] as string[]);
  onMount(async () => {
    try {
      const resp = await getPlatformLogs();
      setLogFiles((resp as string[]).sort());
    } catch (err) {
      handleHttpError(err as Error, t("platform.logs.errors.fetchList.title")!);
    }
  });

  function disable() {
    if (ws) {
      ws.onclose = null;
      ws.close();
    }
    setLogs([]);
    setEnableStreamLogs(false);
  }

  function enable() {
    connect();
    setEnableStreamLogs(true);
  }

  function connect() {
    ws = new WebSocket(`${api_root}/platform/logs/stream?token=${accountStore.token}`);
    setLoading(true);
    ws.onopen = () => {
      setLoading(false);
      setLogs([]);
    };
    ws.onmessage = (event) => {
      const log = JSON.parse(event.data as string) as Log;
      setLogs(logs().concat(log));
      setTimeout(() => {
        bottomDiv!.scrollIntoView({ behavior: "smooth" });
      }, 100);
    };
    ws.onclose = () => {
      addToast({
        level: "error",
        description: t("platform.logs.errors.socketClosed.title")!,
        duration: 5000,
      });
      setTimeout(() => {
        connect();
      }, 5000);
    };
    ws.onerror = () => {
      ws.close();
    };
  }

  onCleanup(disable);

  function getColor(level: string) {
    switch (level) {
      case "INFO":
        return "text-info";
      case "WARN":
        return "text-warning";
      case "ERROR":
        return "text-error";
      case "DEBUG":
        return "text-gray-500";
      default:
        return "text-info";
    }
  }
  function getContentColor(level: string) {
    switch (level) {
      case "INFO":
        return "text-layer-content";
      case "WARN":
        return "text-warning";
      case "ERROR":
        return "text-error";
      case "DEBUG":
        return "opacity-60";
      default:
        return "text-layer-content";
    }
  }
  const time = (ts: string, format: string) => DateTime.fromISO(ts).toFormat(format);
  const matches = createBreakpoints(breakpoints);
  let bottomDiv: HTMLDivElement;
  return (
    <>
      <Title page={t("platform.logs.title")} route="/admin/logs" />
      <div class="flex-1 flex flex-col">
        <div class="sticky top-16 h-16 z-20 backdrop-blur-sm border-b border-b-layer-content/10 flex flex-row space-x-4 items-center px-3 lg:px-6">
          <h1 class="flex-1 font-bold flex flex-row space-x-2 items-center">
            <span class="shrink-0 icon-[fluent--code-20-regular] w-5 h-5" />
            <span>{t("platform.logs.title")}</span>
          </h1>
          <Show when={searchParams.filter}>
            <Button size="sm" onClick={() => setSearchParams({ filter: undefined })}>
              <span class="icon-[fluent--dismiss-16-regular]" />
              <span class="text-error">TRACE: {searchParams.filter}</span>
            </Button>
          </Show>
          <Show when={enableStreamLogs()}>
            <Tag level="success">
              <span>{t("platform.logs.streamingEnabled")}</span>
            </Tag>
          </Show>
          <Button
            onClick={() => (enableStreamLogs() ? disable() : enable())}
            size="sm"
            level={enableStreamLogs() ? "error" : "info"}
          >
            <span>{`${enableStreamLogs() ? t("general.actions.stop.title")! : t("general.actions.start.title")!}`}</span>
          </Button>
        </div>
        <div class="inline-flex flex-row items-center flex-wrap p-3 lg:p-6 !pb-0">
          <For each={logFiles()}>
            {(file) => (
              <DownloadButton
                class="m-1 overflow-hidden"
                size="sm"
                url={`${api_root}/platform/logs`}
                searchParams={{ file }}
                withFileName
                file={file.replace("ret2shell.", "").replace(".log", "")}
                icon="icon-[fluent--folder-zip-16-regular]"
              />
            )}
          </For>
        </div>
        <div class="flex-1 flex flex-col p-3 lg:p-6">
          <For each={logs()}>
            {(log) => (
              <div
                class={clsx(
                  "group min-h-8 flex flex-col h-auto hover:bg-layer-content/15",
                  "px-2 py-1 items-center border-b border-b-layer-content/10 overflow-hidden min-w-0",
                  searchParams.filter && searchParams.filter !== SpanHttp(log)?.event_id && "hidden"
                )}
              >
                <span class="w-full grid grid-cols-[repeat(3,auto)_1fr]">
                  <span class={clsx("w-16 mr-2 inline-block", getColor(log.level))}>{log.level}</span>
                  <span class="mr-2 text-success font-bold" title={log.target}>
                    {log.target}
                  </span>
                  <span class="truncate">
                    {(SpanHttp(log)?.method as string) && (
                      <span class="opacity-60">{SpanHttp(log)?.method as string}&nbsp;</span>
                    )}
                    {(SpanHttp(log)?.uri as string) && (
                      <span class="opacity-60">{SpanHttp(log)?.uri as string}&nbsp;</span>
                    )}
                  </span>
                  <span class="text-right font-bold ml-2 whitespace-nowrap">
                    <span class="font-bold mr-2">
                      <span class="icon-[fluent--person-16-filled] text-primary align-middle w-4 h-4 mr-1" />
                      {SpanUser(log) ?? "GLOBAL"}
                    </span>
                    {(SpanHttp(log)?.from as string) && (
                      <>
                        <span class="icon-[fluent--location-16-filled] text-primary align-middle w-4 h-4 mr-1" />
                        <A
                          href={`/admin/users?filter=${SpanHttp(log)?.from as string}`}
                          target="_blank"
                          rel="noreferrer"
                          class="hover:underline mr-2"
                        >
                          {SpanHttp(log)?.from as string}
                        </A>
                      </>
                    )}
                    {(SpanHttp(log)?.event_id as string) && (
                      <>
                        <span class="icon-[fluent--fire-16-filled] text-primary align-middle w-4 h-4 mr-1" />
                        <A
                          href={`/admin/logs?filter=${SpanHttp(log)?.event_id as string}`}
                          class="hover:underline mr-2"
                        >
                          TRACE
                        </A>
                      </>
                    )}
                    <span class="icon-[fluent--clock-16-filled] text-primary align-middle w-4 h-4 mr-1" />
                    <Switch fallback={time(log.timestamp, "HH:mm:ss")}>
                      <Match when={matches.xl}>{time(log.timestamp, "yyyy-MM-dd HH:mm:ss")}</Match>
                      <Match when={matches.lg}>{time(log.timestamp, "MM-dd HH:mm:ss")}</Match>
                      <Match when={matches.md}>{time(log.timestamp, "yyyy-MM-dd HH:mm:ss")}</Match>
                      <Match when={matches.sm}>{time(log.timestamp, "MM-dd HH:mm:ss")}</Match>
                    </Switch>
                  </span>
                </span>
                <div class="grid grid-cols-[1fr] group-hover:block w-full">
                  <span class={clsx("truncate break-words group-hover:whitespace-normal", getContentColor(log.level))}>
                    {LogField(log)}
                  </span>
                </div>
              </div>
            )}
          </For>
          <Show when={loading()}>
            <div class="h-8 flex flex-row items-center space-x-2 border-b border-b-layer-content/10 overflow-hidden min-w-0">
              <LoadingTips />
            </div>
          </Show>
          <Show when={logs().length === 0}>
            <div class="flex-1 flex flex-col items-center justify-center space-y-8 opacity-60">
              <span class="shrink-0 icon-[fluent--code-20-regular] w-24 h-24" />
              <span>{t("platform.logs.notStreaming")}</span>
              <span>{t("platform.logs.enableStreamWarning")}</span>
            </div>
          </Show>
          <div ref={bottomDiv!} />
        </div>
      </div>
    </>
  );
}
