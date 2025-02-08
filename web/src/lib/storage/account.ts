import { getInstitutes, getProfile } from "@api/account";
import type { Institute } from "@models/institute";
import type { Permission, Token, User } from "@models/user";
import { base64urlnopad } from "@scure/base";
import { makePersisted } from "@solid-primitives/storage";
import { createStore } from "solid-js/store";

export const [accountStore, setAccountStore] = makePersisted(
  createStore({
    id: null as number | null,
    account: null as string | null,
    nickname: null as string | null,
    token: null as string | null,
    info: null as User | null,
    permissions: [] as Permission[],
    institutes: [] as Institute[],
    warnedCodeGeneration: false,
  }),
  { name: "account" }
);

export function storeToken(token: string) {
  setAccountStore({ token });
  const tokenRaw = new TextDecoder().decode(base64urlnopad.decode(token.split(".")[1]));
  const tokenJson = JSON.parse(tokenRaw) as Token;
  setAccountStore({
    id: tokenJson.id,
    account: tokenJson.account,
    nickname: tokenJson.nickname,
    permissions: tokenJson.permissions,
  });
}

export function resetUser() {
  setAccountStore({
    id: null,
    account: null,
    nickname: null,
    token: null,
    info: null,
    permissions: [],
    warnedCodeGeneration: false,
  });
}

export async function refreshUser() {
  if (!accountStore.token) return;
  try {
    setAccountStore({ info: await getProfile() });
  } catch {
    resetUser();
  }
}

export async function refreshInstitutes() {
  try {
    const institutes = await getInstitutes();
    setAccountStore({ institutes });
  } catch {
    // make eslint happy
    setAccountStore({ institutes: [] });
  }
}
