import { luxonReplacer, luxonReviver } from "@models/utils";
import { base64 } from "@scure/base";
import { accountStore, resetUser, storeToken } from "@storage/account";
import { platformStore } from "@storage/platform";
import { experimental_createQueryPersister } from "@tanstack/query-persist-client-core";
import { QueryClient } from "@tanstack/solid-query";
import ky from "ky";

export { handleHttpError, toastError, toastSuccess } from "./utils";

export const api_root = (import.meta.env.VITE_API_ROOT as string) || "/api";

const Ret2StreamTable = "SUCaeck4xrsbgtPwnGY56qpm9vWDIZAKVjlf.HFd,E17Tz0iNQ2yJMLh8OoRuX3B";
const OriginalStreamTable = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const api = ky.extend({
  parseJson: (text) => JSON.parse(text, luxonReviver) as unknown,
  stringifyJson: (data) => {
    let result = JSON.stringify(data, luxonReplacer);
    if (platformStore.enable_ret2codec) {
      result = base64.encode(new TextEncoder().encode(result));
      result = result
        .split("")
        .map((c) => {
          const index = OriginalStreamTable.indexOf(c);
          return index === -1 ? c : Ret2StreamTable[index];
        })
        .join("");
    }
    return result;
  },
  hooks: {
    beforeRequest: [
      (request) => {
        const token = accountStore.token;
        if (token) {
          request.headers.set("Authorization", `Bearer ${token}`);
        }

        if (platformStore.enable_ret2codec && request.headers.get("Content-Type") === "application/json") {
          request.headers.set("X-Original-Content-Type", request.headers.get("Content-Type") || "application/json");
          request.headers.set("Content-Type", "application/x-ret2stream");
        }
      },
    ],
    afterResponse: [
      (_request, _options, response) => {
        if (response.status === 401) {
          resetUser();
        }
        if (response.headers.has("Set-Token")) {
          const token = response.headers.get("Set-Token");
          if (token) {
            storeToken(token);
          }
        }
      },
    ],
  },
});

export default api;

export const inflyClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
      staleTime: 1000 * 5, // 5 seconds
    },
  },
});

export const persister = experimental_createQueryPersister({
  storage: localStorage,
  prefix: "ret2infly",
});
