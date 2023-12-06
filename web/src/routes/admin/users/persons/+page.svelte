<script lang="ts">
  import { getInstituteList, getUserInfo, getUserList } from '$lib/api/v1/user'
  import type { DTColumnAction, DTColumnsDef } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import { i18n } from '$lib/i18n'
  import type { Institute } from '$lib/models/institute'
  import { Permission, permissionToString, type User } from '$lib/models/user'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onDestroy, onMount } from 'svelte'
  import PersonPanel from './PersonPanel.svelte'
  import { page } from '$app/stores'

  let currentPage: number = 1
  let perPage: number = 15
  let total: number = 0
  let loading = false
  let users: User[] = []
  let institutes: Institute[] = []
  let filter: string = ''
  let showPersonPanel = false
  let currentUser: User | null = null

  $: readerUsers = users.map((a) => {
    return {
      ...a,
      name: `${encodeURI(a.name)}|#${encodeURI(a.id.toString())}`,
      permissions: a.permissions
        .filter((p) => p !== Permission.Basic)
        .map((p) => {
          return permissionToString(p)
        })
        .join('|'),
      institute_info: institutes.find((i) => i.id === a.institute_id)?.name || '',
    }
  })
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
      header: $i18n.t('user.name'),
      type: 'plain',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    email: {
      header: $i18n.t('user.email'),
      type: 'plain',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    intro: {
      header: $i18n.t('user.intro'),
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    coverPath: {
      header: $i18n.t('user.coverPath'),
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    institute_info: {
      header: $i18n.t('user.institute_info'),
      type: 'plain',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    institute_id: {
      header: $i18n.t('user.institute_id'),
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    permissions: {
      header: $i18n.t('user.permissions'),
      type: 'tags',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    hidden: {
      header: '',
      type: 'bool',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    banned: {
      header: '',
      type: 'bool',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
  }
  function fetchUsers() {
    loading = true
    getUserList(currentPage, perPage, 'id', filter)
      .then((res) => {
        users = res.users
        total = res.total
        // console.log(users)
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('users.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
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

  $: {
    if (currentPage && !showPersonPanel) {
      fetchUsers()
    }
  }

  let loadingUser = false

  const unsubscribe = page.subscribe((val) => {
    if (val.url.hash && val.url.hash.replace('#', '')) {
      const id = parseInt(val.url.hash.replace('#', ''))
      if (isNaN(id)) {
        loadingUser = false
        showPersonPanel = false
        return
      }
      if (id === currentUser?.id && showPersonPanel === true) {
        return
      }
      loadingUser = true
      showPersonPanel = true
      getUserInfo(id)
        .then((res) => {
          currentUser = res
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('users.fetchInfoFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
        .finally(() => {
          loadingUser = false
        })
    } else {
      loadingUser = false
      showPersonPanel = false
    }
  })

  onDestroy(() => {
    unsubscribe()
  })
</script>

<svelte:head><title>{$i18n.t('admin.userListSettings')} - {$platform.name}</title></svelte:head>
<div class="w-full flex-1 flex flex-col relative">
  {#if !showPersonPanel}
    <div class="w-full flex-1 flex flex-col px-6 lg:px-12">
      <div class="h-16 flex flex-row items-center">
        <h2 class="text-base font-bold flex-1">{$i18n.t('admin.userListSettings')}</h2>
        <div>
          <RxInput
            size="sm"
            placeholder={$i18n.t('admin.filter')}
            icon="icon-[fluent--question-20-regular]"
            bind:value={filter}
          >
            <RxButton
              class="join-item ml-0"
              size="sm"
              on:click={() => {
                currentPage = 1
                fetchUsers()
              }}
            >
              <span class="icon-[fluent--filter-20-regular] w-4 h-4"></span>
            </RxButton>
          </RxInput>
        </div>
      </div>
      <DataTable
        class="flex-1"
        {actions}
        data={readerUsers}
        {colDef}
        bind:page={currentPage}
        {total}
        {loading}
        booleanIconsDef={{
          hidden: {
            true: 'icon-[fluent--eye-off-20-regular] text-warning',
            false: '',
          },
          banned: {
            true: 'icon-[fluent--circle-off-20-regular] text-error',
            false: '',
          },
        }}
      />
    </div>
  {:else}
    <PersonPanel
      user={currentUser}
      loading={loadingUser}
      {institutes}
      class="flex-1"
      on:close={() => {
        window.location.hash = ''
      }}
    />
  {/if}
</div>
