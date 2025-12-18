import type { PlatformLicense } from "@api/platform";
import { makePersisted } from "@solid-primitives/storage";
import { createRoot } from "solid-js";
import { createStore } from "solid-js/store";

export const frontendCompatVersion = import.meta.env.VITE_COMPAT_VERSION as string;

const platformRoot = createRoot(() =>
  makePersisted(
    createStore({
      version: `${frontendCompatVersion}-UNKNOWN-0.0.0`,
      accept_cookies: false,
      under_maintenance: false,
      backend_online: false,
      license: null as null | PlatformLicense,
      enable_ret2codec: null as null | boolean,
      get isOnline() {
        return this.backend_online && !this.under_maintenance;
      },
      get isCompatible() {
        return this.version === frontendCompatVersion;
      },
    }),
    { name: "platform" }
  )
);

export const platformStore = platformRoot[0];
export const setPlatformStore = platformRoot[1];
