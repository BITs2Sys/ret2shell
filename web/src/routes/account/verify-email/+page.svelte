<script lang="ts">
import RxButton from "$lib/components/RxButton.svelte"
import { i18n } from '$lib/i18n'
import { page } from '$app/stores'
import { onMount } from 'svelte'
import { showMessage } from '$lib/stores/toast'
import { goto } from '$app/navigation'
import { verifyEmail } from '$lib/api/account'
import { platform } from '$lib/stores/platform'

onMount(() => {
    const email = $page.url.searchParams.get("email")
    const token = $page.url.searchParams.get("token")
    if (!email || !token) {
        showMessage("error", $i18n.t("email.invalidVerifyLink"), undefined)
        goto("/")
        return
    }
    verifyEmail({email, token})
        .then(() => {
            showMessage("success", $i18n.t("email.verifySuccess"), 5000)
        }).catch((err) => {
            showMessage("error", `${$i18n.t("email.verifyFailed")}: ${err.response?.data}`, undefined)
        }).finally(() => {
            goto("/")
        })
})
</script>

<svelte:head><title>{$i18n.t('account.verifyEmail')} - {$platform.name}</title></svelte:head>

<div class="flex flex-col flex-1">
    <RxButton class="m-6 lg:mt-24" ghost loading>{$i18n.t('account.emailVerifying')}</RxButton>
</div>
