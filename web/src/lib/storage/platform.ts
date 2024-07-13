import type { PlatformLicense } from "@api/platform";
import type { ServerConfig } from "@models/config";
import { makePersisted } from "@solid-primitives/storage";
import { createStore } from "solid-js/store";

export const frontendCompatVersion = import.meta.env.VITE_COMPAT_VERSION as string;

export const [platformStore, setPlatformStore] = makePersisted(
  createStore({
    config: {
      host: location.hostname,
      port: 0,
      external_domain: location.hostname,
      external_https: location.protocol === "https:",
      cors_origins: "*",
      api_base_path: "/api",
      name: null as string | null,
      subject_url: null as string | null,
      subject_info: null as string | null,
      footer_url: null as string | null,
      footer_info: null as string | null,
      record: null as string | null,
      hide_maker: false as boolean | null,
    } as ServerConfig,
    version: `${frontendCompatVersion}-UNKNOWN-0.0.0`,
    accept_cookies: false,
    under_maintenance: false,
    backend_online: false,
    license: null as null | PlatformLicense,
    get isOnline() {
      return this.backend_online && !this.under_maintenance;
    },
    get isCompatible() {
      return this.version === frontendCompatVersion;
    },
  }),
  { name: "platform" }
);
