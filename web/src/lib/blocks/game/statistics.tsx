import { handleHttpError } from "@api";
import { useInstitutes } from "@api/account";
import { useChallenges } from "@api/challenge";
import { type GameStatisticsExport, getGameStatisticsExport, useGame, useGameStatistics } from "@api/game";
import Spin from "@assets/animates/spin";
import logo from "@assets/logo.svg";
import XLSX from "@e965/xlsx";
import { mediaPath } from "@lib/utils/media";
import type { User } from "@models/user";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Chart from "@widgets/chart";
import Divider from "@widgets/divider";
import Select from "@widgets/select";
import { createMemo, createSignal, Show } from "solid-js";

export default function GameStatistics(props: { training?: boolean; gameId: number }) {
  const [selectedInstituteId, setSelectedInstituteId] = createSignal<number | null>(null);
  const game = useGame({ id: () => props.gameId });
  const challenges = useChallenges({ game_id: () => props.gameId });
  const stats = useGameStatistics({
    game_id: () => props.gameId,
    training: () => props.training,
    institute: () => selectedInstituteId() ?? undefined,
  });
  const institutes = useInstitutes();

  const challengeStats = createMemo(() => {
    if (challenges.data && challenges.data[0].length > 0 && stats.data) {
      return challenges.data[0]
        .map((challenge) => ({
          id: challenge.id,
          name: challenge.name,
          submissions: stats.data?.challenge_submissions[challenge.id] || 0,
          solves: stats.data?.challenge_solves[challenge.id] || 0,
        }))
        .sort((a, b) => b.solves - a.solves);
    }
    return [];
  });
  const gameInstitutes = createMemo(() => {
    return institutes.data?.filter((i) => game.data?.access_policy.institutes.includes(i.id)) || [];
  });
  const gameInstitutesSelect = createMemo(() => {
    return gameInstitutes().map((i) => ({
      value: i.id.toString(),
      label: i.name,
      icon: "icon-[fluent--hat-graduation-20-regular] w-5 h-5",
    }));
  });

  const [exporting, setExporting] = createSignal(false);

  async function exportStatisticsJson() {
    if (game.data) {
      setExporting(true);
      try {
        const data = await getGameStatisticsExport(game.data.id, !props.training, selectedInstituteId() ?? undefined);
        // save as json
        const blob = new Blob([JSON.stringify(data)], {
          type: "application/json",
        });
        const url = URL.createObjectURL(blob);
        const link = document.createElement("a");
        link.href = url;
        link.download = `statistics.${gameInstitutes().find((i) => i.id === selectedInstituteId())?.name ?? "general"}.json`;
        link.click();
        URL.revokeObjectURL(url);
      } catch (err) {
        handleHttpError(err as Error, t("game.statistics.errors.export"));
      }
      setExporting(false);
    }
  }

  function _exportStatisticsXlsx(data: GameStatisticsExport) {
    const workbook = XLSX.utils.book_new();
    const statisticsSheet = XLSX.utils.sheet_new();
    const statisticsArr = [];
    for (const [key, value] of Object.entries(data.statistics)) {
      statisticsArr.push([key, value]);
    }
    // add header
    XLSX.utils.sheet_add_aoa(
      statisticsSheet,
      [[gameInstitutes().find((i) => i.id === selectedInstituteId())?.name ?? "general"]],
      { origin: "A1" }
    );
    const max_width = statisticsArr.reduce((max, row) => Math.max(max, (row[0] as string).length), 0);
    statisticsSheet["!cols"] = [{ wch: max_width }];
    XLSX.utils.sheet_add_aoa(statisticsSheet, statisticsArr, { origin: "A2" });
    XLSX.utils.book_append_sheet(workbook, statisticsSheet, "Statistics");

    const challengeStatsHeader = ["Challenge", "Submissions", "Solves"];
    const challengeStatsSheet = XLSX.utils.aoa_to_sheet([challengeStatsHeader]);
    const challengeStatsArr = challengeStats().map((v) => [v.name, v.submissions, v.solves]);
    XLSX.utils.sheet_add_aoa(challengeStatsSheet, challengeStatsArr, {
      origin: "A2",
    });
    XLSX.utils.book_append_sheet(workbook, challengeStatsSheet, "Challenge Stats");

    const scoreboardHeader = [
      "Rank",
      "ID",
      "Team",
      "Tag",
      "Institute",
      "State",
      "Score",
      ...Array.from({ length: game.data?.team_size ?? 0 }, (_, i) => `PLAYER ${i}`),
      ...Array.from({ length: challenges.data?.[0].length ?? 0 }, (_, i) => challenges.data?.[0][i].name),
    ];
    const scoreboardSheet = XLSX.utils.aoa_to_sheet([scoreboardHeader]);
    const scoreboardArr = [];
    function convertTeamState(state: number) {
      switch (state) {
        case 0:
          return "Banned";
        case 1:
          return "Pending";
        case 2:
          return "Hidden";
        case 3:
          return "OK";
      }
    }
    for (const [index, [team, members]] of data.scoreboard.entries()) {
      const paddedMembers: User[] = JSON.parse(JSON.stringify(members));
      while (paddedMembers.length < (game.data?.team_size ?? 0)) {
        paddedMembers.push({
          id: 0,
          account: "",
          nickname: "",
          email: "",
        } as User);
      }
      const row = [
        index + 1,
        team.id,
        team.name,
        team.tag,
        institutes.data?.find((i) => i.id === team.institute_id)?.name ?? "",
        convertTeamState(team.state),
        team.score,
        ...paddedMembers.map((m) => (m.id ? `${m.id}:${m.account} (${m.nickname}) <${m.email}>` : "")),
        ...(challenges.data?.[0].map((c) => (team.history.find((h) => h.challenge_id === c.id) ? "*" : "")) ?? []),
      ];
      scoreboardArr.push(row);
    }
    XLSX.utils.sheet_add_aoa(scoreboardSheet, scoreboardArr, { origin: "A2" });
    XLSX.utils.book_append_sheet(workbook, scoreboardSheet, "Scoreboard");

    const auditHeader = ["Created At", "User", "Team", "Reason"];
    const auditSheet = XLSX.utils.aoa_to_sheet([auditHeader]);
    const auditArr = [];
    for (const audit of data.audits) {
      const row = [audit.created_at.toISO(), audit.user_name, audit.team_name, audit.reason];
      auditArr.push(row);
    }
    XLSX.utils.sheet_add_aoa(auditSheet, auditArr, { origin: "A2" });
    XLSX.utils.book_append_sheet(workbook, auditSheet, "Audits");

    const blob = new Blob([XLSX.write(workbook, { bookType: "xlsx", type: "array" })], {
      type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `statistics.${gameInstitutes().find((i) => i.id === selectedInstituteId())?.name ?? "general"}.xlsx`;
    link.click();
    URL.revokeObjectURL(url);
  }

  async function exportStatisticsXlsx() {
    if (game.data) {
      setExporting(true);
      try {
        _exportStatisticsXlsx(
          await getGameStatisticsExport(game.data.id, !props.training, selectedInstituteId() ?? undefined)
        );
      } catch (err) {
        handleHttpError(err as Error, t("game.statistics.errors.export"));
      }
      setExporting(false);
    }
  }

  return (
    <div class="flex-1 flex flex-col p-3 lg:p-6 gap-3 lg:gap-6 w-full">
      <div class="hidden lg:flex items-center justify-start space-x-12 px-12">
        <img class="w-24 h-24" src={game.data?.logo ? mediaPath(game.data.logo) : logo} alt="CTF" />
        <h1 class="text-5xl font-bold flex-1 truncate">{game.data?.name}</h1>
        <h2 class="text-5xl font-bold">{t("game.statistics.title")}</h2>
      </div>
      <div class="flex flex-row space-x-2 p-3">
        <Show when={!props.training}>
          <Select
            class="flex-1 max-w-64 min-w-0 w-0"
            size="sm"
            placeholder={t("game.statistics.selectInstitute")}
            items={gameInstitutesSelect()}
            onValueChange={(v) => {
              setSelectedInstituteId((v.value.at(0) && Number.parseInt(v.value.at(0)!, 10)) || null);
            }}
          />
        </Show>
        <div class="flex-1" />
        <Button size="sm" onClick={exportStatisticsJson} loading={exporting()} disabled={exporting()}>
          <Show when={!exporting()}>
            <span class="shrink-0 icon-[fluent--open-20-regular] w-5 h-5" />
          </Show>
          <span>JSON</span>
        </Button>
        <Button size="sm" onClick={exportStatisticsXlsx} loading={exporting()} disabled={exporting()}>
          <Show when={!exporting()}>
            <span class="shrink-0 icon-[fluent--open-20-regular] w-5 h-5" />
          </Show>
          <span>XLSX</span>
        </Button>
      </div>
      <Divider />
      <div class="flex flex-col lg:flex-row space-x-2 p-3">
        <div class="lg:flex-1 flex flex-col space-y-2 lg:space-y-4">
          <div class="flex flex-row space-x-4 items-center flex-1">
            <span class="shrink-0 icon-[fluent--emoji-sparkle-20-regular] w-8 h-8 opacity-80" />
            <Show when={!stats.isLoading} fallback={<Spin width={24} height={24} />}>
              <span class="font-bold text-3xl text-info">{stats.data?.total_players}</span>
            </Show>
            <span class="opacity-60">PLAYERS</span>
          </div>
          <div class="flex flex-row space-x-4 items-center flex-1">
            <span class="shrink-0 icon-[fluent--people-team-20-regular] w-8 h-8 opacity-80" />
            <Show when={!stats.isLoading} fallback={<Spin width={24} height={24} />}>
              <span class="font-bold text-3xl text-success">{stats.data?.total_teams}</span>
            </Show>
            <span class="opacity-60">TEAMS</span>
          </div>
        </div>
        <div class="lg:flex-1 flex flex-col space-y-2 lg:space-y-4">
          <div class="flex flex-row space-x-4 items-center flex-1">
            <span class="shrink-0 icon-[fluent--checkmark-starburst-20-regular] w-8 h-8 opacity-80" />
            <Show when={!stats.isLoading} fallback={<Spin width={24} height={24} />}>
              <span class="font-bold text-3xl text-success">{stats.data?.total_solves}</span>
            </Show>
            <span class="opacity-60">SOLVES</span>
          </div>
          <div class="flex flex-row space-x-4 items-center flex-1">
            <span class="shrink-0 icon-[fluent--text-bullet-list-20-regular] w-8 h-8 opacity-80" />
            <Show when={!stats.isLoading} fallback={<Spin width={24} height={24} />}>
              <span class="font-bold text-3xl text-warning">{stats.data?.total_submissions}</span>
            </Show>
            <span class="opacity-60">SUBMITS</span>
          </div>
        </div>
        <div class="lg:flex-1 flex flex-col space-y-2 lg:space-y-4">
          <div class="flex flex-row space-x-4 items-center flex-1">
            <span class="shrink-0 icon-[fluent--code-20-regular] w-8 h-8 opacity-80" />
            <Show when={!stats.isLoading} fallback={<Spin width={24} height={24} />}>
              <span class="font-bold text-3xl text-info">{challenges.data?.[0].filter((c) => !c.hidden).length}</span>
            </Show>
            <span class="opacity-60">PLUBLISHED CHALLS</span>
          </div>
          <div class="flex flex-row space-x-4 items-center flex-1">
            <span class="shrink-0 icon-[fluent--target-edit-20-regular] w-8 h-8 opacity-80" />
            <Show when={!stats.isLoading} fallback={<Spin width={24} height={24} />}>
              <span class="font-bold text-3xl text-info">{challenges.data?.[0].length}</span>
            </Show>
            <span class="opacity-60">TOTAL CHALLS</span>
          </div>
        </div>
      </div>
      <div class="flex flex-col space-y-4">
        <div class="flex flex-row items-center">
          <h2 class="font-bold flex-1 truncate">{t("game.statistics.submission")}</h2>
        </div>
        <div class="h-64 flex flex-row lg:space-x-4">
          <div class="h-full aspect-square flex items-center justify-center">
            <Show when={!stats.isLoading} fallback={<Spin width={24} height={24} />}>
              <Chart
                option={{
                  grid: {
                    left: "16px",
                    right: "16px",
                    bottom: "16px",
                    top: "16px",
                  },
                  toolbox: {},
                  tooltip: {
                    show: true,
                  },
                  series: {
                    type: "sunburst",
                    emphasis: {
                      focus: "ancestor",
                    },
                    data: [
                      {
                        itemStyle: {
                          color: "#17a750",
                        },
                        name: "Solves",
                        value: stats.data?.total_solves ?? 0,
                      },
                      {
                        name: "Fails",
                        value: (stats.data?.total_submissions ?? 0) - (stats.data?.total_solves ?? 0),
                        itemStyle: {
                          color: "#db640e",
                        },
                      },
                    ],
                    label: {
                      show: false,
                    },
                    levels: [
                      {},
                      {
                        r0: "40%",
                        r: "80%",
                      },
                    ],
                  },
                }}
              />
            </Show>
          </div>
          <Show when={!stats.isLoading} fallback={<Spin width={24} height={24} />}>
            <Chart
              class="hidden lg:block flex-1"
              option={{
                grid: {
                  left: "64px",
                  right: "32px",
                  bottom: "32px",
                  top: "48px",
                },
                title: {
                  text: t("game.statistics.challengeStats"),
                  right: "center",
                },
                tooltip: {
                  trigger: "axis",
                  axisPointer: {
                    type: "line",
                    label: {
                      precision: 0,
                    },
                    snap: true,
                  },
                  borderColor: "transparent",
                },
                toolbox: {},
                xAxis: {
                  type: "category",
                  data: challengeStats()?.map((v) => v.name) || [],
                },
                yAxis: {
                  type: "value",
                  min: 0,
                },
                series: [
                  {
                    type: "bar",
                    itemStyle: {
                      normal: {
                        color: "#808080",
                      },
                    },
                    data: challengeStats()?.map((v) => v.submissions) || [],
                    barMaxWidth: 64,
                    barGap: "-100%",
                  },
                  {
                    type: "bar",
                    data: challengeStats()?.map((v) => v.solves) || [],
                    barMaxWidth: 64,
                  },
                ],
              }}
            />
          </Show>
        </div>
      </div>
      <Show
        when={!props.training}
        fallback={
          <div class="flex flex-col space-y-4">
            <div class="flex flex-row items-center">
              <h2 class="font-bold flex-1 truncate">{t("game.statistics.players")}</h2>
            </div>
            <div class="h-64 flex flex-row lg:space-x-4">
              <div class="h-full aspect-square flex items-center justify-center">
                <Show when={!stats.isLoading} fallback={<Spin width={24} height={24} />}>
                  <Chart
                    class="hidden lg:block"
                    option={{
                      grid: {
                        left: "16px",
                        right: "16px",
                        bottom: "16px",
                        top: "16px",
                      },
                      toolbox: {},
                      tooltip: {
                        show: true,
                      },
                      series: {
                        type: "sunburst",
                        emphasis: {
                          focus: "ancestor",
                        },
                        data: Object.entries(stats.data?.institute_players || {}).map(([key, value]) => ({
                          name: institutes.data?.find((v) => v.id === Number.parseInt(key, 10))?.name || key,
                          value,
                        })),
                        label: {
                          show: false,
                        },
                        levels: [
                          {},
                          {
                            r0: "40%",
                            r: "80%",
                          },
                        ],
                      },
                    }}
                  />
                </Show>
              </div>
              <Show when={!stats.isLoading} fallback={<Spin width={24} height={24} />}>
                <Chart
                  class="flex-1"
                  option={{
                    grid: {
                      left: "64px",
                      right: "32px",
                      bottom: "32px",
                      top: "48px",
                    },
                    title: {
                      text: t("game.statistics.institutePlayers"),
                      right: "center",
                    },
                    tooltip: {
                      trigger: "axis",
                      axisPointer: {
                        type: "line",
                        label: {
                          precision: 0,
                        },
                        snap: true,
                      },
                      borderColor: "transparent",
                    },
                    toolbox: {},
                    xAxis: {
                      type: "category",
                      data: Object.entries(stats.data?.institute_players || {})
                        .map(([i, _]) => institutes.data?.find((v) => v.id === Number.parseInt(i, 10))?.name)
                        .concat(t("game.statistics.others")),
                    },
                    yAxis: {
                      type: "value",
                      min: 0,
                    },
                    series: {
                      type: "bar",
                      data: Object.entries(stats.data?.institute_players || {})
                        .map(([_, v]) => v)
                        .concat(
                          (stats.data?.total_players ?? 0) -
                            Object.values(stats.data?.institute_players || {}).reduce((a, b) => a + b, 0)
                        ),
                      barMaxWidth: 64,
                    },
                  }}
                />
              </Show>
            </div>
          </div>
        }
      >
        <div class="flex flex-col space-y-4">
          <div class="flex flex-row items-center">
            <h2 class="font-bold flex-1 truncate">{t("game.statistics.teams")}</h2>
          </div>
          <div class="h-64 flex flex-row lg:space-x-4">
            <div class="h-full aspect-square flex items-center justify-center">
              <Show when={!stats.isLoading} fallback={<Spin width={24} height={24} />}>
                <Chart
                  class="block"
                  option={{
                    grid: {
                      left: "16px",
                      right: "16px",
                      bottom: "16px",
                      top: "16px",
                    },
                    tooltip: {
                      show: true,
                    },
                    toolbox: {},
                    series: {
                      type: "sunburst",
                      emphasis: {
                        focus: "ancestor",
                      },
                      label: {
                        show: false,
                      },
                      data: Object.entries(stats.data?.institute_teams || {}).map(([key, value]) => ({
                        name: institutes.data?.find((v) => v.id === Number.parseInt(key, 10))?.name || key,
                        value,
                      })),
                      levels: [
                        {},
                        {
                          r0: "40%",
                          r: "80%",
                        },
                      ],
                    },
                  }}
                />
              </Show>
            </div>
            <Show when={!stats.isLoading} fallback={<Spin width={24} height={24} />}>
              <Chart
                class="hidden lg:block flex-1"
                option={{
                  grid: {
                    left: "64px",
                    right: "32px",
                    bottom: "32px",
                    top: "48px",
                  },
                  title: {
                    text: t("game.statistics.instituteTeams"),
                    right: "center",
                  },
                  tooltip: {
                    trigger: "axis",
                    axisPointer: {
                      type: "line",
                      label: {
                        precision: 0,
                      },
                      snap: true,
                    },
                    borderColor: "transparent",
                  },
                  toolbox: {},
                  xAxis: {
                    type: "category",
                    data: Object.entries(stats.data?.institute_teams || {})
                      .map(([i, _]) => institutes.data?.find((v) => v.id === Number.parseInt(i, 10))?.name)
                      .concat(t("game.statistics.others")),
                  },
                  yAxis: {
                    type: "value",
                    min: 0,
                  },
                  series: {
                    type: "bar",
                    data: Object.entries(stats.data?.institute_teams || {})
                      .map(([_, v]) => v)
                      .concat(
                        (stats.data?.total_teams ?? 0) -
                          Object.values(stats.data?.institute_teams || {}).reduce((a, b) => a + b, 0)
                      ),
                    barMaxWidth: 64,
                  },
                }}
              />
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
}
