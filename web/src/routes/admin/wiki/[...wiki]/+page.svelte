<script lang="ts">
  import { i18n } from '$lib/i18n'
  import { page } from '$app/stores'
  import type { Wiki } from '$lib/models/wiki'
  import { createWiki, deleteWiki, editWiki, getWiki } from '$lib/api/wiki'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onDestroy } from 'svelte'
  import RxCodearea from '$lib/components/RxCodearea.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import { goto } from '$app/navigation'
  import { blur } from 'svelte/transition'
  import { platform } from '$lib/stores/platform'

  let loading = true
  let isCreate = false
  let wiki: Wiki = {
    id: 0,
    title: '',
    content: '',
    parent: null,
    published_at: 0,
    updated_at: 0,
    author_id: 0,
  }

  let editedWiki: Wiki = {
    id: 0,
    title: '',
    content: '',
    parent: null,
    published_at: 0,
    updated_at: 0,
    author_id: 0,
  }

  let deleteModalOpened = false
  const unsubscribe = page.subscribe((value) => {
    const arr = value.params['wiki'].split('/')
    // console.log(arr)
    const id = parseInt(arr[arr.length - 1])
    if (isNaN(id)) {
      loading = false
      return
    }
    // console.log(id)
    if (id === 0) {
      isCreate = true
      loading = false
      //   console.log('create')
      wiki = {
        id: 0,
        title: '',
        content: '',
        parent: null,
        published_at: 0,
        updated_at: 0,
        author_id: 0,
      }
      editedWiki = {
        id: 0,
        title: '',
        content: '',
        parent: null,
        published_at: 0,
        updated_at: 0,
        author_id: 0,
      }
      return
    }
    loading = true
    if (id > 0) {
      getWiki(id)
        .then((data) => {
          wiki = data

          isCreate = $page.url.hash.replace('#', '') === 'create'
          if (isCreate) {
            editedWiki = {
              id: 0,
              title: '',
              content: '',
              parent: null,
              published_at: 0,
              updated_at: 0,
              author_id: 0,
            }
          } else {
            editedWiki = structuredClone(wiki)
          }
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('wiki.fetchContentFailed')}: ${(err as AxiosError).response?.data}`)
        })
        .finally(() => {
          loading = false
        })
    } else {
      loading = false
    }
  })

  function handleDeleteWiki() {
    deleteWiki(wiki.id)
      .then(() => {
        showMessage('success', $i18n.t('announcement.deleteSuccess'), 5000)
        deleteModalOpened = false
        if (wiki.parent != null) {
          let pathArr = $page.url.pathname.split('/').filter((item) => item !== '')
          pathArr.pop()
          goto(`/${pathArr.join('/')}#edit`, { replaceState: true })
        } else {
          goto('/admin/wiki/0#create', { replaceState: true })
        }
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('announcement.deleteFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
  }

  onDestroy(unsubscribe)

  function createOrSaveWikiItem() {
    if (isCreate) {
      if (editedWiki.title.trim() === '') {
        showMessage('error', $i18n.t('wiki.titleEmpty'), 5000)
        return
      } else if (editedWiki.content.trim() === '') {
        showMessage('error', $i18n.t('wiki.contentEmpty'), 5000)
        return
      }
      if (wiki.id !== 0) editedWiki.parent = wiki.id
      else editedWiki.parent = null
      createWiki(editedWiki)
        .then((data) => {
          showMessage('success', $i18n.t('wiki.createSuccess'), 5000)
          let pathArr = $page.url.pathname
            .replace($page.url.hash, '')
            .split('/')
            .filter((item) => item !== '')
          if (parseInt(pathArr[pathArr.length - 1]) === 0) pathArr.pop()
          const pathName = pathArr.join('/')
          goto(`/${pathName}/${data.id}#edit`, { replaceState: true })
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('wiki.createFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    } else {
      if (editedWiki.title.trim() === '') {
        showMessage('error', $i18n.t('wiki.titleEmpty'), 5000)
        return
      } else if (editedWiki.content.trim() === '') {
        showMessage('error', $i18n.t('wiki.contentEmpty'), 5000)
        return
      }
      editWiki(wiki.id, editedWiki)
        .then(() => {
          showMessage('success', $i18n.t('wiki.updateSuccess'), 5000)
          const pathName = $page.url.pathname.replace($page.url.hash, '')
          goto(`${pathName}#edit`, { replaceState: true })
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('wiki.updateFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    }
  }
  function deleteWikiItem() {
    deleteModalOpened = true
  }
</script>

<svelte:head><title>{editedWiki.title} - {$platform.name}</title></svelte:head>

<div class="flex flex-col min-h-full relative backdrop-blur">
  <div class="h-16 flex-shrink-0 flex flex-row items-center px-2 border-b border-b-base-content/5 relative">
    <RxInput ghost bind:value={editedWiki.title}></RxInput>
    <RxButton class="join-item" ghost level="info" on:click={createOrSaveWikiItem}>
      {#if isCreate}
        <span class="icon-[fluent--add-20-regular] w-5 h-5"></span>
        <span>{$i18n.t('admin.create')}</span>
      {:else}
        <span class="icon-[fluent--save-20-regular] w-5 h-5"></span>
        <span>{$i18n.t('admin.update')}</span>
      {/if}
    </RxButton>
    {#if !isCreate}
      <RxButton class="join-item" ghost level="error" on:click={deleteWikiItem}>
        <span class="icon-[fluent--delete-20-regular] w-5 h-5"></span>
      </RxButton>
    {/if}
  </div>
  <RxCodearea class="flex-1" lang="markdown" bind:value={editedWiki.content} droppable={true}></RxCodearea>
  {#if deleteModalOpened}
    <div
      class="fixed top-0 left-0 w-full h-full bg-base-100/60 z-50 flex flex-col items-center justify-center"
      transition:blur={{ amount: 20, duration: 300 }}
    >
      <div class="rounded-box p-4 flex flex-col space-y-4 bg-neutral w-80">
        <h1 class="text-base font-bold">
          {$i18n.t('admin.deleteWikiConfirm', { item: decodeURI(wiki.title.split('|')[0]) })}
        </h1>
        <div class="flex flex-row justify-end space-x-4">
          <RxButton size="sm" on:click={() => (deleteModalOpened = false)}>{$i18n.t('form.cancel')}</RxButton>
          <RxButton size="sm" level="error" on:click={handleDeleteWiki}>{$i18n.t('form.confirm')}</RxButton>
        </div>
      </div>
    </div>
  {/if}
  {#if loading}
    <div
      class="absolute top-0 left-0 w-full h-full z-20 backdrop-blur flex flex-row justify-center items-center"
      transition:blur={{ amount: 20, duration: 300 }}
    >
      <span class="loading loading-spinner loading-sm" />
    </div>
  {/if}
</div>
