<script lang="ts">
  import { getInstituteList, getUserList } from '$lib/api/user'
  import type { DTColumnAction, DTColumnsDef, DTDataEntry } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import { i18n } from '$lib/i18n'
  import type { Institute } from '$lib/models/institute'
  import { Permission, permissionToString, type User } from '$lib/models/user'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onMount } from 'svelte'

  let page: number = 1
  let perPage: number = 15
  let total: number = 0
  let loading = false
  let users: User[] = []
  let institutes: Institute[] = []

  $: readerUsers = users.map((a) => {
    return {
      ...a,
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
      icon: 'icon-[fluent--edit-16-regular]',
      label: '',
      level: 'info',
      type: 'link',
      href: '#{id}',
    },
    {
      icon: 'icon-[fluent--delete-16-regular]',
      label: '',
      level: 'error',
      type: 'button',
      onClick: (data: DTDataEntry) => {
        // popup delete modal
      },
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
    getUserList(page, perPage)
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
    fetchUsers()
  })

  $: {
    if (page) {
      fetchUsers()
    }
  }
</script>

<div class="w-full flex-1 flex flex-col px-6 lg:px-12">
  <div class="h-16 flex flex-row items-center">
    <h2 class="text-base font-bold flex-1">{$i18n.t('admin.userListSettings')}</h2>
  </div>
  <DataTable
    class="flex-1"
    {actions}
    data={readerUsers}
    {colDef}
    bind:page
    {total}
    {loading}
    booleanIconsDef={{
      hidden: {
        true: 'icon-[fluent--eye-off-16-regular] text-warning',
        false: '',
      },
      banned: {
        true: 'icon-[fluent--circle-off-16-regular] text-error',
        false: '',
      },
    }}
  />
</div>
