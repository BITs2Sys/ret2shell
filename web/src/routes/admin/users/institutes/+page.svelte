<script lang="ts">
  import { getInstituteList } from '$lib/api/user'
  import type { DTColumnAction, DTColumnsDef } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import { i18n } from '$lib/i18n'
  import type { Institute } from '$lib/models/institute'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onMount } from 'svelte'

  let page: number = 1
  let total: number = 0
  let loading = false
  let institutes: Institute[] = []
  let actions: DTColumnAction[] = [
    {
      icon: 'icon-[fluent--edit-20-regular]',
      label: '',
      level: 'info',
      type: 'link',
      href: '#{id}',
    },
  ]
  let colDef: DTColumnsDef = {
    id: {
      header: 'ID',
      dimmed: true,
      type: 'number',
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    name: {
      header: $i18n.t('institute.name'),
      type: 'plain',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    description: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    validator: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    data: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    logo: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
  }
  function fetchInstitutes() {
    loading = true
    getInstituteList()
      .then((res) => {
        institutes = res
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('institutes.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }

  onMount(() => {
    fetchInstitutes()
  })
</script>

<svelte:head><title>{$i18n.t('admin.instituteListSettings')} - {$platform.name}</title></svelte:head>

<div class="w-full flex-1 flex flex-col px-6 lg:px-12">
  <div class="h-16 flex flex-row items-center">
    <h2 class="text-base font-bold flex-1">{$i18n.t('admin.instituteListSettings')}</h2>
  </div>
  <DataTable class="flex-1" {actions} data={institutes} {colDef} bind:page {total} {loading} />
</div>
