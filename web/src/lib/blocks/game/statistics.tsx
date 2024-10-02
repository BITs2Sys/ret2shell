import { getGameStatistics } from "@api/game";
import { gameStore } from "@storage/game";
import { addToast } from "@storage/toast";
import { t } from "@storage/theme";
import type { HTTPError } from "ky";
import { createEffect, createMemo, createSignal, Show, untrack } from "solid-js";
import { platformStore } from "@storage/platform";
import { Title } from "@storage/header";
import logo from "@assets/logo.svg";
import { mediaPath } from "@lib/utils/media";
import Divider from "@widgets/divider";
import Spin from "@assets/animates/spin";
import Chart from "@widgets/chart";
import { accountStore, refreshInstitutes } from "@storage/account";
import { challengeStore, refreshChallenges } from "@storage/challenge";

export default function GameStatistics(props: {
  inGame?: boolean;
}) {
  const [loading, setLoading] = createSignal(false);
  const [stats, setStats] = createSignal(
    null as {
      total_players: number;
      institute_players: { [key: number]: number };
      total_teams: number;
      total_passed_teams: number;
      institute_teams: { [key: number]: number };
      total_submissions: number;
      total_solves: number;
      challenge_submissions: { [key: number]: number };
      challenge_solves: { [key: number]: number };
    } | null
  );

  createEffect(() => {
    if (gameStore.current) {
      untrack(() => {
        setLoading(true);
        refreshChallenges();
        refreshInstitutes();
        getGameStatistics(gameStore.current!.id, props.inGame)
          .then((data) => {
            setStats(data);
          })
          .catch((err: HTTPError) => {
            err.response.text().then((text) => {
              addToast({
                level: "error",
                description: `${t("game.fetchFailed")}: ${text}`,
                duration: 5000,
              });
            });
          })
          .finally(() => {
            setLoading(false);
          });
      });
    }
  });

  const challenges = createMemo(() => {
    if (challengeStore) {
      return challengeStore.challenges;
    }
    return [];
  });

  const challengeStats = createMemo(() => {
    if (challenges().length > 0 && stats()) {
      return challenges()
        .map((challenge) => ({
          id: challenge.id,
          name: challenge.name,
          submissions: stats()?.challenge_submissions[challenge.id] || 0,
          solves: stats()?.challenge_solves[challenge.id] || 0,
        }))
        .sort((a, b) => b.solves - a.solves);
    }
    return [];
  });

  return (
    <>
      <Title title={`${t("game.statistics.title")} - ${platformStore.config.name || t("platform.name")}`} />
      <div class="flex-1 flex flex-col p-3 lg:p-6 gap-3 lg:gap-6 w-full">
        <div class="hidden xl:flex items-center justify-start space-x-12 px-12">
          <img class="w-24 h-24" src={gameStore.current?.logo ? mediaPath(gameStore.current!.logo) : logo} alt="CTF" />
          <h1 class="text-5xl font-bold flex-1 truncate">{gameStore.current?.name}</h1>
          <h2 class="text-5xl font-bold">{t("game.statistics.title")}</h2>
        </div>
        <Divider />
        <div class="flex flex-col space-y-4">
          <div class="flex flex-row items-center">
            <h2 class="font-bold flex-1 truncate">{t("game.statistics.players")}</h2>
            <h3 class="font-bold text-primary">{stats()?.total_players}</h3>
          </div>
          <div class="h-64 flex flex-row lg:space-x-4">
            <div class="h-full aspect-square flex items-center justify-center">
              <Show when={!loading() && stats()} fallback={<Spin width={24} height={24} />}>
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
                      data: Object.entries(stats()!.institute_players).map(([key, value]) => ({
                        name: accountStore.institutes.find((v) => v.id === Number.parseInt(key))?.name || key,
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
            <Show when={!loading() && stats()} fallback={<Spin width={24} height={24} />}>
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
                    data: Object.entries(stats()!.institute_players)
                      .map(([i, _]) => accountStore.institutes.find((v) => v.id === Number.parseInt(i))?.name)
                      .concat(t("game.statistics.others")!),
                  },
                  yAxis: {
                    type: "value",
                    min: 0,
                  },
                  series: {
                    type: "bar",
                    data: Object.entries(stats()!.institute_players)
                      .map(([_, v]) => v)
                      .concat(
                        stats()!.total_players - Object.values(stats()!.institute_players).reduce((a, b) => a + b, 0)
                      ),
                    barMaxWidth: 64,
                  },
                }}
              />
            </Show>
          </div>
        </div>
        <div class="flex flex-col space-y-4">
          <div class="flex flex-row items-center">
            <h2 class="font-bold flex-1 truncate">{t("game.statistics.teams")}</h2>
            <h3 class="font-bold text-primary">{stats()?.total_teams}</h3>
          </div>
          <div class="h-64 flex flex-row lg:space-x-4">
            <div class="h-full aspect-square flex items-center justify-center">
              <Show when={!loading() && stats()} fallback={<Spin width={24} height={24} />}>
                <Chart
                  class="hidden lg:block"
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
                      data: Object.entries(stats()!.institute_teams).map(([key, value]) => ({
                        name: accountStore.institutes.find((v) => v.id === Number.parseInt(key))?.name || key,
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
            <Show when={!loading() && stats()} fallback={<Spin width={24} height={24} />}>
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
                    data: Object.entries(stats()!.institute_teams)
                      .map(([i, _]) => accountStore.institutes.find((v) => v.id === Number.parseInt(i))?.name)
                      .concat(t("game.statistics.others")!),
                  },
                  yAxis: {
                    type: "value",
                    min: 0,
                  },
                  series: {
                    type: "bar",
                    data: Object.entries(stats()!.institute_teams)
                      .map(([_, v]) => v)
                      .concat(
                        stats()!.total_players - Object.values(stats()!.institute_teams).reduce((a, b) => a + b, 0)
                      ),
                    barMaxWidth: 64,
                  },
                }}
              />
            </Show>
          </div>
        </div>
        <div class="flex flex-col space-y-4">
          <div class="flex flex-row items-center">
            <h2 class="font-bold flex-1 truncate">{t("game.statistics.submissions")}</h2>
          </div>
          <div class="h-64 flex flex-row lg:space-x-4">
            <div class="h-full aspect-square flex items-center justify-center">
              <Show when={!loading() && stats()} fallback={<Spin width={24} height={24} />}>
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
                          value: stats()?.total_solves ?? 0,
                        },
                        {
                          name: "Fails",
                          value: (stats()?.total_submissions ?? 0) - (stats()?.total_solves ?? 0),
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
            <Show when={!loading() && stats()} fallback={<Spin width={24} height={24} />}>
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
                    text: t("game.statistics.challengeSolveDetails"),
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
      </div>
    </>
  );
}
