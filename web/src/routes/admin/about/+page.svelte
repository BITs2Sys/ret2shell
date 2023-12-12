<script lang="ts">
  import { getPlatformLicense, getPlatformVersion } from '$lib/api/v1/platform'
  import LogoFull from '$lib/assets/logo-full.svg'
  import RxTag from '$lib/components/RxTag.svelte'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'

  let version = ''

  getPlatformVersion()
    .then((res) => (version = res))
    .catch((err) => {
      showMessage(
        'error',
        $i18n.t('platform.failedToFetchPlatformVersion') + ': ' + (err as AxiosError).response?.data,
        5000
      )
    })

  let license = {
    issuer: '',
    date: '',
    website: '',
  }

  getPlatformLicense()
    .then((res) => (license = res))
    .catch((err) => {
      showMessage(
        'error',
        $i18n.t('platform.failedToFetchPlatformVersion') + ': ' + (err as AxiosError).response?.data,
        5000
      )
    })

  $: isDevVersion = version.includes('*')
</script>

<svelte:head><title>{$i18n.t('admin.routes.about')} - {$platform.name}</title></svelte:head>
<div class="flex-1 flex flex-col p-6 lg:p-12 space-y-4 items-center max-w-7xl self-center">
  <h1 class="text-xl flex flex-row items-center space-x-4">
    <img src={LogoFull} width={480} alt="" />
  </h1>
  <h2 class="font-bold text-xl">Version {version.replace('*', '')}</h2>
  <div class="flex flex-row items-center space-x-2 bg-info/10 p-4 rounded-lg backdrop-blur w-full">
    <span class="icon-[fluent--document-ribbon-20-regular] w-5 h-5 text-success flex-shrink-0" />
    <span>
      Licensed to <a class="hover:underline" href="https://{license.website}">{license.issuer}</a>
      &lt;{license.website}&gt;, will expire at {license.date}
    </span>
  </div>
  {#if isDevVersion}
    <div class="flex flex-row items-center space-x-2 bg-warning/10 p-4 rounded-lg backdrop-blur w-full">
      <span class="icon-[fluent--warning-20-regular] w-5 h-5 text-warning flex-shrink-0" />
      <span>
        {$i18n.t('about.devVersionWarning')}
      </span>
    </div>
  {/if}
  <div class="divider"></div>
  <h2 class="opacity-80 text-base font-bold">CONTRIBUTORS</h2>
  <div class="flex flex-row flex-wrap">
    <RxTag class="m-1" level="success">
      <a class="hover:underline" href="https://github.com/Reverier-Xu">Reverier-Xu</a>
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/DX39061">DX39061</a>
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/xiaoxi404">xiaoxi404</a>
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/ZeroAurora">ZeroAurora</a>
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/frankli0324">frankli0324</a>
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/orangecheers-x">orangecheers-x</a>
    </RxTag>
  </div>
  <div class="divider"></div>
  <h2 class="opacity-80 text-base font-bold">THIRD PARTY LIBRARIES</h2>
  <div class="flex flex-row flex-wrap justify-center">
    <RxTag class="m-1" level="success">
      <a class="hover:underline" href="https://tokio.rs">tokio</a>
      <img src="https://img.shields.io/badge/MIT-blue.svg" alt="MIT" />
    </RxTag>
    <RxTag class="m-1" level="success">
      <a class="hover:underline" href="https://svelte.dev">svelte</a>
      <img src="https://img.shields.io/badge/MIT-blue.svg" alt="MIT" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/tokio-rs/axum">axum</a>
      <img src="https://img.shields.io/badge/MIT-blue.svg" alt="MIT" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/tokio-rs/tracing">tracing</a>
      <img src="https://img.shields.io/badge/MIT-blue.svg" alt="MIT" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/lettre/lettre">lettre</a>
      <img src="https://img.shields.io/badge/MIT-blue.svg" alt="MIT" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/kube-rs/kube">kube</a>
      <img src="https://img.shields.io/badge/Apache_2.0-blue.svg" alt="Apache 2.0" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://hyper.rs/">hyper</a>
      <img src="https://img.shields.io/badge/MIT-blue.svg" alt="MIT" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/tower-rs/tower">tower</a>
      <img src="https://img.shields.io/badge/MIT-blue.svg" alt="MIT" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://www.sea-ql.org/SeaORM/">SeaORM</a>
      <img src="https://img.shields.io/badge/MIT-blue.svg" alt="MIT" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/briansmith/ring">ring</a>
      <img src="https://img.shields.io/badge/multiple-blue.svg" alt="multiple" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://serde.rs/">serde</a>
      <img src="https://img.shields.io/badge/MIT-blue.svg" alt="MIT" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/chronotope/chrono">chrono</a>
      <img src="https://img.shields.io/badge/MIT_or_Apache_2.0-blue.svg" alt="MIT / Apache 2.0" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/BurntSushi/aho-corasick">aho-corasick</a>
      <img src="https://img.shields.io/badge/Unlicense-blue.svg" alt="Unlicense" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/kornelski/deunicode">deunicode</a>
      <img src="https://img.shields.io/badge/custom-blue.svg" alt="custom" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/ret2shell/wsrx">wsrx</a>
      <img src="https://img.shields.io/badge/MIT-blue.svg" alt="MIT" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/ret2shell/wsvc">wsvc</a>
      <img src="https://img.shields.io/badge/MIT-blue.svg" alt="MIT" />
    </RxTag>
    <RxTag class="m-1">
      <a class="hover:underline" href="https://github.com/Reverier-Xu/biosvg">biosvg</a>
      <img src="https://img.shields.io/badge/MPL_2.0-blue.svg" alt="MPL 2.0" />
    </RxTag>
  </div>
  <div class="divider"></div>
  <h2 class="opacity-80 text-base font-bold">PLATFORMS</h2>
  <div class="flex flex-row flex-wrap justify-center">
    <RxTag class="m-1" level="success">
      <a class="hover:underline" href="https://www.postgresql.org/">PostgreSQL</a>
      <img src="https://img.shields.io/badge/PostgreSQL-blue.svg" alt="PostgreSQL" />
    </RxTag>
    <RxTag class="m-1" level="success">
      <a class="hover:underline" href="https://nats.io">NATS</a>
      <img src="https://img.shields.io/badge/Apache_2.0-blue.svg" alt="Apache 2.0" />
    </RxTag>
    <RxTag class="m-1" level="success">
      <a class="hover:underline" href="https://redis.io">Redis</a>
      <img src="https://img.shields.io/badge/BSD_3_Clause-blue.svg" alt="BSD-3-Clause" />
    </RxTag>
    <RxTag class="m-1" level="success">
      <a class="hover:underline" href="https://k3s.io">K3S</a>
      <img src="https://img.shields.io/badge/Apache_2.0-blue.svg" alt="Apache 2.0" />
    </RxTag>
  </div>
  <div class="flex-1"></div>
  <p class="opacity-60 text-base font-bold">{$i18n.t('platform.about.tip1')}</p>
</div>
