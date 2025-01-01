import { useLocation } from "@solidjs/router";
import { createEffect, untrack } from "solid-js";
import { platformStore } from "./platform";
import { t } from "./theme";

class RouteHeader {
  title: string;
  path: string;
  subRoutes: RouteHeader[];
  constructor() {
    this.title = "";
    this.path = "";
    this.subRoutes = [];
  }

  /// returns: [parentRoute, exactRoute]
  findRoute(subPath: string[]): [RouteHeader, RouteHeader | null] {
    if (subPath.length === 0) {
      return [this, this];
    }
    const subRoute = this.subRoutes.find((r) => r.path === subPath[0]);
    if (subRoute) {
      return subRoute.findRoute(subPath.slice(1));
    }
    return [this, null];
  }

  insertRoute(subPath: string[], title: string) {
    if (subPath.length === 0) {
      this.title = title;
      return;
    }
    const subRoute = this.subRoutes.find((r) => r.path === subPath[0]);
    if (subRoute) {
      subRoute.insertRoute(subPath.slice(1), title);
    } else {
      const newRoute = new RouteHeader();
      newRoute.path = subPath[0];
      newRoute.insertRoute(subPath.slice(1), title);
      this.subRoutes.push(newRoute);
    }
  }
}

export const headerStore = new RouteHeader();

export function Title(props: { title: string }) {
  const path = useLocation().pathname;
  const pathArr = path.split("/");
  const [, exactRoute] = headerStore.findRoute(pathArr);
  if (exactRoute) {
    exactRoute.title = props.title;
  } else {
    headerStore.insertRoute(pathArr, props.title);
  }
  return null;
}

export function setupTitleResolver() {
  const watchedLocation = useLocation();
  createEffect(() => {
    let path = watchedLocation.pathname;
    untrack(() => {
      if (path.endsWith("/")) path = path.slice(0, path.length - 1);
      const pathArr = path.split("/");
      const [parentRoute, exactRoute] = headerStore.findRoute(pathArr);
      document.title = exactRoute?.title || parentRoute.title || platformStore.config.name || t("platform.name")!;
    });
  });
}
