import { api_root, handleHttpError } from "@api";
import { getPlatformLogs, type Log, queryPlatformLog } from "@api/platform";
import DownloadButton from "@blocks/download-button";
import { createBreakpoints } from "@solid-primitives/media";
import { A, useSearchParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { breakpoints, t } from "@storage/theme";
import { useQuery } from "@tanstack/solid-query";
import Button from "@widgets/button";
import Input from "@widgets/input";
import LoadingTips from "@widgets/loading-tips";
import Select from "@widgets/select";
import clsx from "clsx";
import { DateTime } from "luxon";
import { createSignal, For, Match, onCleanup, Show, Switch } from "solid-js";

function LogField(log: Log) {
  if (!log) return null;
  return (
    <For each={Object.entries(log)}>
      {([key, value]) =>
        key.startsWith("fields.") && (
          <>
            <span class="italic opacity-60">{key.replace("fields.", "")}</span>
            <span class="opacity-60">=</span>
            <span>{value as string}&nbsp;&nbsp;</span>
          </>
        )
      }
    </For>
  );
}

function DataSpanField(log: Log) {
  if (!log.span) return null;
  return (
    <For each={Object.entries(log.span)}>
      {([key, value]) =>
        key.startsWith("span.data-") && (
          <>
            <span class="italic opacity-60">{key.replace("span.data-", "")}</span>
            <span class="opacity-60">=</span>
            <span>{value as string}&nbsp;&nbsp;</span>
          </>
        )
      }
    </For>
  );
}

export default function() {
  const [searchParams, setSearchParams] = useSearchParams();

  const logFiles = useQuery(() => ({
    queryKey: ["platform", "logs", "files"],
    queryFn: async () => (await getPlatformLogs()).sort((a, b) => b.localeCompare(a)),
    onError: (err: Error) => {
      handleHttpError(err, t("platform.logs.errors.fetchList.title")!);
    },
  }));

  const logs = useQuery(() => ({
    queryKey: [
      "platform",
      "logs",
      "stream",
      searchParams.started_at,
      searchParams.ended_at,
      searchParams.limit,
      searchParams.level,
      searchParams.trace,
      searchParams.from,
      searchParams.account,
    ].filter((i) => i),
    queryFn: async () => {
      return (
        await queryPlatformLog({
          started_at: searchParams.started_at
            ? DateTime.fromSeconds(Number.parseInt(searchParams.started_at as string, 10))
            : DateTime.now().minus({ day: 1 }),
          ended_at: searchParams.ended_at
            ? DateTime.fromSeconds(Number.parseInt(searchParams.ended_at as string, 10))
            : DateTime.now(),
          limit: searchParams.limit ? Number.parseInt((searchParams.limit as string) || "NaN", 10) : 20,
          level: searchParams.level as string | undefined,
          trace: searchParams.trace as string | undefined,
          from: searchParams.from as string | undefined,
          account: searchParams.account as string | undefined,
          query: searchParams.query as string | undefined,
        })
      ).sort((a, b) => {
        return DateTime.fromISO(a._time).toMillis() - DateTime.fromISO(b._time).toMillis();
      });
    },
    onError: (err: Error) => {
      handleHttpError(err, t("platform.logs.errors.fetchLogs.title")!);
    },
  }));

  const [enableTimer, setEnableTimer] = createSignal(false);

  const timer = setInterval(() => {
    if (enableTimer()) {
      setSearchParams({
        started_at: Math.floor(DateTime.now().minus({ day: 1 }).toSeconds()).toString(),
        ended_at: Math.floor(DateTime.now().toSeconds()).toString(),
        limit: searchParams.limit,
        level: searchParams.level,
        trace: searchParams.trace,
        from: searchParams.from,
        account: searchParams.account,
      });
    }
  }, 5000);

  onCleanup(() => clearInterval(timer));

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

  const time = (ts: string, format: string) => DateTime.fromISO(ts).toFormat(format);
  const matches = createBreakpoints(breakpoints);
  let bottomDiv: HTMLDivElement;
  return (
    <>
      <Title page={t("platform.logs.title")} route="/admin/logs" />
      <div class="flex-1 flex flex-col">
        <div class="sticky top-16 h-16 z-20 backdrop-blur-sm border-b border-b-layer-content/10 flex flex-row space-x-4 items-center pr-4 pl-3 lg:pl-6">
          <h1 class="flex-1 font-bold flex flex-row space-x-2 items-center">
            <span class="shrink-0 icon-[fluent--code-20-regular] w-5 h-5" />
            <span>{t("platform.logs.title")}</span>
          </h1>
          <Input
            size="sm"
            class="w-48"
            icon={<span class="icon-[fluent--flash-16-regular]" />}
            value={searchParams.trace ?? ""}
            placeholder="Trace ID"
            onInput={(e) => setSearchParams({ trace: e.currentTarget.value })}
          />
          <Input
            size="sm"
            class="w-48"
            icon={<span class="icon-[fluent--person-16-regular]" />}
            value={searchParams.account ?? ""}
            placeholder="Account"
            onInput={(e) => setSearchParams({ account: e.currentTarget.value })}
          />
          <Input
            size="sm"
            class="w-48"
            icon={<span class="icon-[fluent--location-16-regular]" />}
            value={searchParams.from ?? ""}
            placeholder="IP"
            onInput={(e) => setSearchParams({ from: e.currentTarget.value })}
          />
          <Select
            class="w-36"
            placeholder="LEVEL"
            size="sm"
            items={[
              { label: "ALL", value: "" },
              { label: "DEBUG", value: "DEBUG" },
              { label: "INFO", value: "INFO" },
              { label: "WARN", value: "WARN" },
              { label: "ERROR", value: "ERROR" },
            ]}
            value={[(searchParams.limit as string) ?? "20"]}
            onValueChange={(val) => setSearchParams({ level: val.value.at(0) })}
          />
          <Select
            class="w-36"
            placeholder="LIMIT"
            size="sm"
            items={[
              { label: "10", value: "10" },
              { label: "20", value: "20" },
              { label: "50", value: "50" },
              { label: "100", value: "100" },
              { label: "1000", value: "1000" },
            ]}
            value={[(searchParams.level as string) ?? ""]}
            onValueChange={(val) => setSearchParams({ limit: val.value.at(0) })}
          />
          <Button
            square
            size="sm"
            level={enableTimer() ? "error" : "success"}
            onClick={() => setEnableTimer(!enableTimer())}
          >
            <span class={clsx(enableTimer() ? "icon-[fluent--stop-16-regular]" : "icon-[fluent--play-16-regular]")} />
          </Button>
        </div>
        <div class="inline-flex flex-row items-center flex-wrap p-3 lg:p-6 !pb-0">
          <For each={logFiles.data}>
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
          <For each={logs.data}>
            {(log) => (
              <div
                class={clsx(
                  "group min-h-8 flex flex-col h-auto",
                  "px-2 py-1 items-center border-b border-b-layer-content/10 overflow-hidden min-w-0"
                )}
              >
                <span class="w-full grid grid-cols-[repeat(3,auto)_1fr]">
                  <span class={clsx("w-16 mr-2 inline-block", getColor(log.level))}>{log.level}</span>
                  <span class="mr-2 text-success font-bold" title={log.target}>
                    {log.target}
                  </span>
                  <span class={clsx("truncate", getColor(log.level))}>
                    {(log["span.method"] as string) && <span>{log["span.method"] as string}&nbsp;</span>}
                    {(log["span.uri"] as string) && <span>{log["span.uri"] as string}&nbsp;</span>}
                  </span>
                  <span class="text-right font-bold ml-2 whitespace-nowrap">
                    {(log["span.user-account"] as string) && (
                      <span class="font-bold mr-2">
                        <span class="icon-[fluent--person-16-filled] text-primary align-middle w-4 h-4 mr-1" />
                        <A href={`/admin/logs?account=${log["span.user-account"]}`} class="hover:underline mr-2">
                          {log["span.user-account"] as string}
                        </A>
                        <A href={`/admin/users?user=${log["span.user-id"]}`}>
                          <span class="icon-[fluent--open-12-regular] w-3 h-3 align-top" />
                        </A>
                      </span>
                    )}
                    {(log["span.from"] as string) && (
                      <>
                        <span class="icon-[fluent--location-16-filled] text-primary align-middle w-4 h-4 mr-1" />
                        <A href={`/admin/logs?from=${log["span.from"] as string}`} class="hover:underline mr-2">
                          {log["span.from"] as string}
                        </A>
                        <A href={`/admin/users?filter=${log["span.from"]}`}>
                          <span class="icon-[fluent--open-12-regular] w-3 h-3 align-top" />
                        </A>
                      </>
                    )}
                    {(log["span.trace"] as string) && (
                      <>
                        <span class="icon-[fluent--fire-16-filled] text-primary align-middle w-4 h-4 mr-1" />
                        <A href={`/admin/logs?trace=${log["span.trace"] as string}`} class="hover:underline mr-2">
                          TRACE
                        </A>
                      </>
                    )}
                    <span class="icon-[fluent--clock-16-filled] text-primary align-middle w-4 h-4 mr-1" />
                    <Switch fallback={time(log._time, "HH:mm:ss")}>
                      <Match when={matches.xl}>{time(log._time, "yyyy-MM-dd HH:mm:ss")}</Match>
                      <Match when={matches.lg}>{time(log._time, "MM-dd HH:mm:ss")}</Match>
                      <Match when={matches.md}>{time(log._time, "yyyy-MM-dd HH:mm:ss")}</Match>
                      <Match when={matches.sm}>{time(log._time, "MM-dd HH:mm:ss")}</Match>
                    </Switch>
                  </span>
                </span>
                <div class="grid grid-cols-[1fr] group-hover:block w-full">
                  <span class={clsx("truncate break-words group-hover:whitespace-normal")}>
                    <span>{log._msg}&nbsp;&nbsp;</span>
                    {LogField(log)}
                    {DataSpanField(log)}
                  </span>
                </div>
              </div>
            )}
          </For>
          <Show when={logs.isLoading || logFiles.isLoading}>
            <div class="h-8 flex flex-row items-center space-x-2 border-b border-b-layer-content/10 overflow-hidden min-w-0">
              <LoadingTips />
            </div>
          </Show>
          <Show when={logs.data?.length === 0}>
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
