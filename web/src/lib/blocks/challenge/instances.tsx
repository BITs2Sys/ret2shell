import { api_root } from "@api";
import { getRegistryConfig, getRegistryImageTags, getRegistryRepositories } from "@api/cluster";
import { deleteChallengeEnv, getChallengeInstance, updateChallengeEnv } from "@api/game";
import { Popover as ArkPopover } from "@ark-ui/solid";
import UploadButton from "@blocks/upload-button";
import type { Challenge, ChallengeImage } from "@models/challenge";
import type { RegistryConfig } from "@models/config";
import { createForm, pattern, required, setValue, setValues } from "@modular-forms/solid";
import { challengeStore, refreshChallengeAssets } from "@storage/challenge";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Checkbox from "@widgets/checkbox";
import Dialog from "@widgets/dialog";
import Input from "@widgets/input";
import LoadingTips from "@widgets/loading-tips";
import Popover from "@widgets/popover";
import Select from "@widgets/select";
import Slider from "@widgets/slider";
import type { Pod } from "kubernetes-types/core/v1";
import type { HTTPError } from "ky";
import { For, Show, createEffect, createSignal, untrack } from "solid-js";

function CreateForm(fnProps: {
  repos: string[];
  onDone?: () => void;
}) {
  const [loading, setLoading] = createSignal(false);
  const [registryConfig, setRegistryConfig] = createSignal<RegistryConfig | null>(null);
  getRegistryConfig()
    .then(setRegistryConfig)
    .catch((err: HTTPError) => {
      err.response.text().then((text) => {
        addToast({
          level: "error",
          description: `${t("game.challenge.fetchEnvRegistryConfigFailed")}: ${text}`,
          duration: 5000,
        });
      });
    });
  const [tags, setTags] = createSignal<string[]>([]);
  const [form, { Form, Field }] = createForm<ChallengeImage>();
  setValue(form, "cpu", 0.5);
  setValue(form, "mem", "128Mi");
  const [searchedRepo, setSearchedRepo] = createSignal("");
  const [selected, setSelected] = createSignal(false);
  function fetchTags(repo: string) {
    setLoading(true);
    getRegistryImageTags(repo)
      .then((tags) => {
        setTags(tags);
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("game.challenge.fetchEnvImagesFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => {
        setLoading(false);
      });
  }
  const [adding, setAdding] = createSignal(false);
  function onSubmit(result: ChallengeImage) {
    setAdding(true);
    updateChallengeEnv(challengeStore!.current!.game_id, challengeStore!.current!.id, {
      internet: challengeStore.env?.internet || false,
      images: [...(challengeStore.env?.images || []), result],
    })
      .then(() => {
        addToast({
          level: "success",
          description: t("game.challenge.addEnvSuccess")!,
          duration: 5000,
        });
        setValues(form, {
          name: "",
          tag: "",
          cpu: 0.5,
          mem: "128Mi",
          port: null,
          service_type: null,
          description: "",
        });
        refreshChallengeAssets();
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("game.challenge.addEnvFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => {
        setAdding(false);
        fnProps.onDone?.();
      });
  }
  return (
    <Form onSubmit={onSubmit} class="flex flex-col space-y-2">
      <div class="flex flex-row space-x-2">
        <Field
          name="name"
          validate={[
            required(t("game.challenge.envContainerNameRequired")!),
            pattern(/^[a-z\-]{3,40}$/, t("game.challenge.envContainerNameFormat")!),
          ]}
        >
          {(field, props) => (
            <Input
              class="flex-1"
              icon={<span class="icon-[fluent--flag-20-regular] w-5 h-5" />}
              title={t("game.challenge.envContainerName")}
              placeholder={t("game.challenge.envContainerName")}
              {...props}
              value={field.value}
              error={field.error}
              required
            />
          )}
        </Field>
        <Field name="tag" validate={[required(t("game.challenge.envContainerTagRequired")!)]}>
          {(field, props) => (
            <>
              <input hidden {...props} value={field.value} class="hidden" />
              <div class="flex-1 relative">
                <ArkPopover.Root
                  autoFocus={false}
                  open={!!searchedRepo() && !selected()}
                  closeOnInteractOutside={false}
                >
                  <ArkPopover.Anchor class="w-full">
                    <Input
                      class="w-full"
                      icon={<span class="icon-[fluent--flag-20-regular] w-5 h-5" />}
                      title={t("game.challenge.envContainerTag")}
                      placeholder={t("game.challenge.envContainerTag")}
                      error={field.error}
                      required
                      value={searchedRepo()}
                      onInput={(e) => {
                        setSearchedRepo(e.target.value);
                        setSelected(false);
                        setValue(form, "tag", "");
                      }}
                    />
                  </ArkPopover.Anchor>
                  <ArkPopover.Positioner class="w-full">
                    <ArkPopover.Content class="popover card w-full z-50">
                      <div class="card-content p-2 flex flex-col space-y-2">
                        <Show when={loading()}>
                          <LoadingTips />
                        </Show>
                        <For each={fnProps.repos.filter((repo) => repo.includes(searchedRepo()))}>
                          {(repo) => (
                            <Button
                              class="btn-sm"
                              ghost
                              justify="start"
                              type="button"
                              onClick={() => {
                                setSearchedRepo(repo);
                                setSelected(true);
                                fetchTags(repo);
                              }}
                            >
                              <span class="icon-[fluent--beaker-20-regular] w-5 h-5" />
                              <span>{repo}</span>
                            </Button>
                          )}
                        </For>
                      </div>
                    </ArkPopover.Content>
                  </ArkPopover.Positioner>
                </ArkPopover.Root>
              </div>
              <Select
                label={t("game.challenge.envContainerTagVersion")!}
                class="flex-1"
                error={field.error}
                placeholder={t("game.challenge.selectEnvContainerTagVersion")}
                items={
                  tags().map((tag) => ({
                    value: tag,
                    label: tag,
                    icon: "icon-[fluent--tag-20-regular]",
                  })) || []
                }
                inputProps={props}
                onValueChange={(e) => {
                  setValue(form, "tag", `${registryConfig()?.external}/${searchedRepo()}:${e.value.at(0)}`);
                }}
              />
            </>
          )}
        </Field>
      </div>
      <div class="flex flex-row space-x-2">
        <Field name="description">
          {(field, props) => (
            <Input
              class="flex-1"
              icon={<span class="icon-[fluent--text-20-regular] w-5 h-5" />}
              title={t("game.challenge.envContainerDescription")}
              placeholder={t("game.challenge.envContainerDescription")}
              {...props}
              value={field.value || undefined}
            />
          )}
        </Field>
        <div class="flex flex-row space-x-2 flex-1">
          <Field name="service_type">
            {(field, props) => (
              <Select
                label={t("game.challenge.envContainerServiceType")}
                class="flex-1"
                placeholder={t("game.challenge.selectEnvContainerServiceType")}
                items={[
                  { value: "http", label: "HTTP", icon: "icon-[fluent--globe-20-regular]" },
                  { value: "tcp", label: "TCP", icon: "icon-[fluent--globe-20-regular]" },
                ]}
                value={field.value ? [field.value as string] : undefined}
                inputProps={props}
                onValueChange={(e) => {
                  setValue(form, "service_type", (e.value[0] || null) as "http" | "tcp" | null);
                }}
              />
            )}
          </Field>
          <Field name="port" type="number">
            {(field, props) => (
              <Input
                class="flex-1"
                icon={<span class="icon-[fluent--cloud-link-20-regular] w-5 h-5" />}
                title={t("game.challenge.envContainerPort")}
                placeholder={t("game.challenge.envContainerPort")}
                type="number"
                {...props}
                value={field.value || undefined}
              />
            )}
          </Field>
        </div>
      </div>
      <div class="flex flex-row space-x-2">
        <Field name="cpu" type="number" validate={[required(t("game.challenge.envContainerCpuRequired")!)]}>
          {(field, props) => (
            <Slider
              class="flex-1"
              label={t("game.challenge.envContainerCpu")}
              max={4}
              min={0.1}
              step={0.1}
              inputProps={props}
              name={field.name}
              value={[field.value || 0.1]}
            />
          )}
        </Field>
        <Field name="mem" validate={[required(t("game.challenge.envContainerMemRequired")!)]}>
          {(field, props) => (
            <>
              <Slider
                class="flex-1"
                label={`${t("game.challenge.envContainerMem")} (MB)`}
                max={4096}
                min={32}
                step={32}
                name={field.name}
                value={[Number.parseInt(field.value?.replace("Mi", "") || "32") || 32]}
                onValueChange={(e) => {
                  setValue(form, "mem", `${e.value[0]}Mi`);
                }}
              />
              <input hidden {...props} value={field.value} class="hidden" />
            </>
          )}
        </Field>
      </div>
      <Button type="submit" level="primary" class="!mt-4" loading={adding()} disabled={adding()}>
        {t("form.add")}
      </Button>
    </Form>
  );
}

function InstanceList() {
  const [pods, setPods] = createSignal<Pod[]>([]);
  createEffect(() => {
    if (challengeStore.current) {
      untrack(() => {
        getChallengeInstance(challengeStore!.current!.game_id, challengeStore!.current!.id)
          .then(setPods)
          .catch((err: HTTPError) => {
            err.response.text().then((text) => {
              addToast({
                level: "error",
                description: `${t("game.challenge.fetchEnvInstancesFailed")}: ${text}`,
                duration: 5000,
              });
            });
          });
      });
    }
  });
  return (
    <ul class="flex flex-col">
      <For
        each={pods()}
        fallback={
          <div class="h-12 flex flex-row space-x-2 items-center opacity-60 border-b border-b-layer-content/10">
            <span class="icon-[fluent--emoji-sad-20-regular] w-5 h-5" />
            <span>{t("game.challenge.noEnvInstances")}</span>
          </div>
        }
      >
        {(pod) => (
          <li class="h-12 flex flex-row space-x-2 items-center border-b border-b-layer-content/10">
            <span class="icon-[fluent--cube-20-regular] w-5 h-5" />
            <span>{pod.metadata?.name}</span>
            <span class="flex-1" />
            <span class="opacity-60">{pod.status?.phase}</span>
          </li>
        )}
      </For>
    </ul>
  );
}

export default function (_props: {
  onStateChange?: (challenge?: Challenge) => void;
  inGame?: boolean;
}) {
  const [repos, setRepos] = createSignal<string[]>([]);
  function fetchRepos() {
    getRegistryRepositories()
      .then((repos) => {
        setRepos(repos);
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("game.challenge.fetchEnvImagesFailed")}: ${text}`,
            duration: 5000,
          });
        });
      });
  }
  createEffect(() => {
    if (challengeStore.current) {
      untrack(fetchRepos);
    }
  });
  function onToggleInternet() {
    updateChallengeEnv(challengeStore!.current!.game_id, challengeStore!.current!.id, {
      internet: !challengeStore.env?.internet,
      images: challengeStore.env?.images || [],
    })
      .then(() => {
        addToast({
          level: "success",
          description: t("game.challenge.toggleEnvInternetSuccess")!,
          duration: 5000,
        });
        refreshChallengeAssets();
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("game.challenge.toggleEnvInternetFailed")}: ${text}`,
            duration: 5000,
          });
        });
      });
  }
  function onDeleteImage(name: string) {
    updateChallengeEnv(challengeStore!.current!.game_id, challengeStore!.current!.id, {
      internet: challengeStore.env?.internet || false,
      images: challengeStore.env?.images?.filter((image) => image.name !== name) || [],
    })
      .then(() => {
        addToast({
          level: "success",
          description: t("game.challenge.deleteEnvImageSuccess")!,
          duration: 5000,
        });
        refreshChallengeAssets();
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("game.challenge.deleteEnvImageFailed")}: ${text}`,
            duration: 5000,
          });
        });
      });
  }
  function onDeleteEnv() {
    deleteChallengeEnv(challengeStore!.current!.game_id, challengeStore!.current!.id)
      .then(() => {
        addToast({
          level: "success",
          description: t("game.challenge.deleteEnvSuccess")!,
          duration: 5000,
        });
        refreshChallengeAssets();
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("game.challenge.deleteEnvFailed")}: ${text}`,
            duration: 5000,
          });
        });
      });
  }
  const [formOpen, setFormOpen] = createSignal(false);
  return (
    <div class="flex-1 flex flex-col space-y-2 p-3 lg:p-6">
      <header class="h-12 border-b border-b-layer-content/15 flex flex-row items-center space-x-2 font-bold">
        <span class="icon-[fluent--settings-20-regular] w-5 h-5 flex-shrink-0" />
        <span class="flex-1 text-start">{t("game.challenge.envImages")}</span>
        <span class="font-bold">{t("game.challenge.uploadImageToRegistry")}:</span>
        <UploadButton
          size="sm"
          url={`${api_root}/cluster/repo`}
          onDone={() => {
            addToast({
              level: "success",
              description: t("game.challenge.uploadImageToRegistrySuccess")!,
              duration: 5000,
            });
            fetchRepos();
          }}
        />
        <Dialog
          size="sm"
          btnContent={<span>{t("game.challenge.addEnvImage")}</span>}
          stretched
          open={formOpen()}
          onOpenChange={(details) => {
            setFormOpen(details.open);
          }}
          onClick={() => {
            setFormOpen(true);
          }}
        >
          <CreateForm
            repos={repos()}
            onDone={() => {
              setFormOpen(false);
            }}
          />
        </Dialog>
        <Show when={challengeStore.env}>
          <Popover level="error" size="sm" btnContent={<span>{t("game.challenge.deleteEnv")}</span>}>
            <Card contentClass="p-2 flex flex-col space-x-2 max-w-96">
              <span class="inline-block space-x-2">
                <span class="icon-[fluent--warning-20-regular] w-5 h-5 text-warning align-middle" />
                <span>{t("game.challenge.deleteEnvTips")}</span>
              </span>
              <Button level="primary" size="sm" class="self-end" onClick={onDeleteEnv}>
                {t("platform.accept")}
              </Button>
            </Card>
          </Popover>
        </Show>
      </header>
      <Checkbox
        checked={challengeStore.env?.internet}
        onChange={() => {
          onToggleInternet();
        }}
      >
        <span class="flex-1 text-start">{t("game.challenge.envHasInternet")}</span>
      </Checkbox>

      <For
        each={challengeStore.env?.images || []}
        fallback={
          <div class="h-12 flex flex-row space-x-2 items-center opacity-60 border-b border-b-layer-content/10">
            <span class="icon-[fluent--emoji-sad-20-regular] w-5 h-5" />
            <span>{t("game.challenge.noEnvImages")}</span>
          </div>
        }
      >
        {(image) => (
          <div class="py-2 flex flex-col space-y-1 border-b border-b-layer-content/10">
            <div class="flex flex-row space-x-2 items-center">
              <span class="icon-[fluent--flag-20-regular] w-5 h-5" />
              <span>{image.name}</span>
              <span class="opacity-60 flex-1 text-end truncate" title={image.tag}>
                {image.tag}
              </span>
              <Popover
                level="error"
                ghost
                size="sm"
                square
                btnContent={<span class="icon-[fluent--delete-20-regular] w-5 h-5" />}
              >
                <Card contentClass="p-2 flex flex-col space-y-2 max-w-96">
                  <span class="inline-block space-x-2">
                    <span class="icon-[fluent--warning-20-regular] w-5 h-5 text-warning align-middle" />
                    <span>{t("game.challenge.deleteEnvImageTips")}</span>
                  </span>
                  <Button level="primary" size="sm" class="self-end" onClick={() => onDeleteImage(image.name)}>
                    {t("platform.accept")}
                  </Button>
                </Card>
              </Popover>
            </div>
            <div class="flex flex-row space-x-2 items-center opacity-80">
              <span class="icon-[fluent--cloud-link-20-regular] w-5 h-5" />
              <span class="text-warning font-bold">
                {image.service_type}:{image.port}
              </span>
              <span>({image.description})</span>
              <span class="flex-1" />
              <span class="icon-[fluent--engine-20-regular] w-5 h-5" />
              <span class="font-bold opacity-60">CPU: {image.cpu}</span>
              <span class="icon-[fluent--box-20-regular] w-5 h-5" />
              <span class="font-bold opacity-60">Memory: {image.mem}</span>
            </div>
          </div>
        )}
      </For>
      <Show when={challengeStore.env}>
        <header class="h-12 border-b border-b-layer-content/15 flex flex-row items-center space-x-2 font-bold">
          <span class="icon-[fluent--settings-20-regular] w-5 h-5 flex-shrink-0" />
          <span class="flex-1 text-start">{t("game.challenge.envInstances")}</span>
        </header>
        <InstanceList />
      </Show>
    </div>
  );
}
