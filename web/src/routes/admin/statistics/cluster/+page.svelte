<script lang="ts">
  import { getPlatformClusterInfo } from '$lib/api/v1/platform'
  import { i18n } from '$lib/i18n'
  import type { ClusterInfo } from '$lib/models/config'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onMount } from 'svelte'
  import Engine from '$lib/assets/animates/engine.svelte'
  import Compass from '$lib/assets/animates/compass.svelte'
  import KubenertesLogo from '$lib/assets/brands/kubernetes.svelte'
  import RxTag from '$lib/components/RxTag.svelte'
  import RxPopup from '$lib/components/RxPopup.svelte'
  import '$lib/styles/transitions.scss'
  import { blur } from 'svelte/transition'
  import { platform } from '$lib/stores/platform'

  let clusterInfo: ClusterInfo | null = null
  let loading = true
  let working = false

  onMount(() => {
    getPlatformClusterInfo()
      .then((resp) => {
        clusterInfo = resp
        working = true
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('platform.fetchStatFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        working = false
      })
      .finally(() => {
        loading = false
      })
  })

  $: since = clusterInfo?.configs.items.find((item) => item.data?.since !== undefined)?.data?.since
  $: clusterDNS = clusterInfo?.configs.items.find((item) => item.data?.clusterDNS !== undefined)?.data?.clusterDNS
  $: clusterDomain = clusterInfo?.configs.items.find((item) => item.data?.clusterDomain !== undefined)?.data
    ?.clusterDomain
</script>

<svelte:head><title>{$i18n.t('admin.routes.cluster')} - {$platform.name}</title></svelte:head>
<div class="flex-1 flex flex-col p-6 space-y-2 relative">
  {#if !loading}
    <h1 class="text-3xl font-bold flex flex-row space-x-4 items-center">
      {#if working}
        <KubenertesLogo class="animate-helm" level="active" width={96} height={96} />
      {:else}
        <KubenertesLogo level="down" width={96} height={96} />
      {/if}
      <div class="flex flex-col space-y-2">
        {#if working}
          <span>
            <span>Kubernetes v{clusterInfo?.version.major}.{clusterInfo?.version.minor}</span>
            <span class="opacity-40">@</span>
            <span class="opacity-80">{clusterInfo?.default_namespace}</span>
          </span>
          <div class="text-base flex flex-row flex-wrap opacity-60">
            <span>
              Version: {clusterInfo?.version.gitVersion}-{clusterInfo?.version.goVersion}-{clusterInfo?.version
                .platform}
            </span>
          </div>
        {:else}
          <span>{$i18n.t('platform.clusterIsDown')}</span>
        {/if}
      </div>
      <div class="flex-1"></div>
      <Compass width={128} height={128} />
    </h1>
  {/if}
  <div class="divider"></div>
  <h2 class="font-bold text-base">{$i18n.t('admin.cluster.basicInfo')}</h2>
  <div class="flex flex-row space-x-6 items-center">
    <span>{$i18n.t('admin.cluster.since')}: {since}</span>
    <span class="flex-1"></span>
    <span>{$i18n.t('admin.cluster.clusterDNS')}: {clusterDNS}</span>
    <span>{$i18n.t('admin.cluster.clusterDomain')}: {clusterDomain}</span>
  </div>
  <div class="divider"></div>
  <h2 class="font-bold text-base">{$i18n.t('admin.cluster.activeNodes')}</h2>
  <div class="flex flex-row flex-wrap">
    {#if clusterInfo?.nodes.items}
      {#each clusterInfo.nodes.items as item}
        <RxPopup
          name={item.metadata.name}
          popupWidth="auto"
          placement="bottom-start"
          class="h-auto w-auto bg-neutral/40 border-base-content/5 backdrop-blur m-2 p-2"
          event="click-blur"
        >
          <div slot="button" class="flex flex-row space-x-2 p-2 items-center">
            <div class="text-success"><Engine width={64} height={64} /></div>
            <div class="flex flex-col font-normal justify-center space-y-2">
              <h2 class="font-bold text-start text-lg">{item.metadata.name}</h2>
              <p class="normal-case opacity-60">{item.status.nodeInfo.osImage}</p>
            </div>
          </div>
          <div class="p-6 rounded-box bg-neutral">
            <div class="flex flex-row items-center p-2 space-x-2 border-b border-b-base-content/5">
              <div class="h-2 w-2 bg-success rounded-full"></div>
              <h2 class="font-bold">{item.metadata.name}</h2>
              <div class="flex-1"></div>
              <div class="opacity-60">{item.metadata.uid}</div>
            </div>
            <div class="flex flex-row flex-wrap p-2">
              <RxTag
                class="m-1"
                label={`${item.status.nodeInfo.operatingSystem}-${item.status.nodeInfo.kernelVersion}`}
              />
              <RxTag class="m-1" label={item.status.nodeInfo.architecture} />
              <RxTag class="m-1" label={item.status.nodeInfo.osImage} />
            </div>
            <table class="table-auto table table-sm">
              {#each item.status.addresses as address}
                <tr>
                  <td class="font-bold opacity-60">{address.type}</td>
                  <td>{address.address}</td>
                </tr>
              {/each}
              <tr>
                <td class="font-bold opacity-60">Pod CIDR</td>
                <td>{item.spec.podCIDR}</td>
                <td class="font-bold opacity-60">Provider ID</td>
                <td>{item.spec.providerID}</td>
              </tr>
              <tr>
                <td class="font-bold opacity-60">Boot ID</td>
                <td>{item.status.nodeInfo.bootID}</td>
                <td class="font-bold opacity-60">Machine ID</td>
                <td>{item.status.nodeInfo.machineID}</td>
              </tr>
              <tr>
                <td class="font-bold opacity-60">Kubelet</td>
                <td>{item.status.nodeInfo.kubeletVersion}</td>
                <td class="font-bold opacity-60">Kube Proxy</td>
                <td>{item.status.nodeInfo.kubeProxyVersion}</td>
              </tr>
              <tr>
                <td class="font-bold opacity-60">Container Runtime</td>
                <td>{item.status.nodeInfo.containerRuntimeVersion}</td>
              </tr>
              <tr>
                <td class="font-bold opacity-60">CPU Allocatable</td>
                <td>{item.status.allocatable.cpu}</td>
                <td class="font-bold opacity-60">CPU Capacity</td>
                <td>{item.status.capacity.cpu}</td>
              </tr>
              <tr>
                <td class="font-bold opacity-60">Memory Allocatable</td>
                <td>{item.status.allocatable.memory}</td>
                <td class="font-bold opacity-60">Memory Capacity</td>
                <td>{item.status.capacity.memory}</td>
              </tr>
              <tr>
                <td class="font-bold opacity-60">Pods Allocatable</td>
                <td>{item.status.allocatable.pods}</td>
                <td class="font-bold opacity-60">Pods Capacity</td>
                <td>{item.status.capacity.pods}</td>
              </tr>
            </table>
          </div>
        </RxPopup>
      {/each}
    {/if}
  </div>
  {#if loading}
    <div
      class="absolute top-0 left-0 w-full h-full z-20 backdrop-blur flex flex-row justify-center items-center"
      transition:blur={{ amount: 20, duration: 300 }}
    >
      <span class="loading loading-spinner loading-sm" />
    </div>
  {/if}
</div>
